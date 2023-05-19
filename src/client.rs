use std::{collections::HashMap, path::PathBuf};

use kilexpr::{
    parser::{Layout, Parameters},
    LayoutGenerator, Rect,
};
use wayland_client::{
    backend::ObjectId,
    protocol::{
        wl_output::{self, WlOutput},
        wl_registry::{self, WlRegistry},
    },
    Dispatch, Proxy,
};

use crate::protocol::{
    river_layout_manager_v3::RiverLayoutManagerV3,
    river_layout_v3::{self, RiverLayoutV3},
};

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct OutputId(ObjectId);

impl OutputId {
    pub fn new(output: &WlOutput) -> OutputId {
        OutputId(output.id())
    }
}

pub struct LayoutManager {
    /// Command Tags.
    tags: u32,
    /// Path of the layout file.
    path: PathBuf,
    /// The list of layouts.
    layouts: HashMap<String, Layout>,
    /// The output configuration.
    outputs: HashMap<OutputId, Output>,
    /// The river_layout_manager_v3 global
    proxy: Option<RiverLayoutManagerV3>,
}

impl Default for LayoutManager {
    fn default() -> Self {
        let mut home = std::env::var("HOME").expect("Failed to found the HOME path!");
        home.push_str("/.config/river/layout.kl");
        Self::new(home)
    }
}

impl LayoutManager {
    pub fn new(path: String) -> Self {
        let mut this = Self {
            tags: u32::MAX,
            path: PathBuf::from(path),
            layouts: HashMap::new(),
            outputs: HashMap::new(),
            proxy: None,
        };
        this.load_layouts();
        this
    }
    pub fn load_layouts(&mut self) {
        match std::fs::read_to_string(self.path.as_path()) {
            Ok(s) => match kilexpr::parse(&s) {
                Ok(parser) => self.layouts = parser.vars,
                Err(err) => eprintln!("{:?}: {:?}", err.kind.cursor(&s), err),
            },
            Err(err) => println!("{:?}", err),
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Output {
    tags: [Tag; 32],
}

#[derive(Debug, Clone, PartialEq)]
pub struct Tag {
    layout: String,
    padding: i32,
    params: Parameters,
}

impl Default for Tag {
    fn default() -> Self {
        Tag {
            layout: String::from("default"),
            padding: 0,
            params: Parameters::default(),
        }
    }
}

pub struct TagIter(u32);

impl TagIter {
    pub fn new(tags: u32) -> Self {
        Self(tags)
    }
}

impl Iterator for TagIter {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.gt(&0).then_some(self.0.trailing_zeros()).map(|tag| {
            self.0 = self.0 ^ 1 << tag;
            tag as usize
        })
    }
}

impl Dispatch<WlRegistry, ()> for LayoutManager {
    fn event(
        state: &mut Self,
        registry: &WlRegistry,
        event: <WlRegistry as Proxy>::Event,
        _: &(),
        _: &wayland_client::Connection,
        qh: &wayland_client::QueueHandle<Self>,
    ) {
        match event {
            wl_registry::Event::Global {
                name,
                interface,
                version,
            } => match interface.as_str() {
                "wl_output" => {
                    registry.bind::<wl_output::WlOutput, _, Self>(name, version, qh, ());
                }
                "river_layout_manager_v3" => {
                    state.proxy =
                        Some(registry.bind::<RiverLayoutManagerV3, _, Self>(name, version, qh, ()));
                }
                _ => {}
            },
            _ => {}
        }
    }
}

impl Dispatch<WlOutput, ()> for LayoutManager {
    fn event(
        state: &mut Self,
        output: &WlOutput,
        event: <WlOutput as wayland_client::Proxy>::Event,
        _: &(),
        _: &wayland_client::Connection,
        qhandle: &wayland_client::QueueHandle<Self>,
    ) {
        if let wl_output::Event::Done = event {
            state
                .outputs
                .insert(OutputId::new(output), Output::default());
            state
                .proxy
                .as_ref()
                .expect("Compositor does not support river_layout_v3!")
                .get_layout(output, String::from("kile"), qhandle, OutputId::new(output));
        }
    }
}

impl Dispatch<RiverLayoutV3, OutputId> for LayoutManager {
    fn event(
        state: &mut Self,
        proxy: &RiverLayoutV3,
        event: <RiverLayoutV3 as Proxy>::Event,
        output: &OutputId,
        _: &wayland_client::Connection,
        _: &wayland_client::QueueHandle<Self>,
    ) {
        match event {
            river_layout_v3::Event::LayoutDemand {
                view_count,
                usable_width,
                usable_height,
                tags,
                serial,
            } => {
                state.tags = tags;
                let rect = Rect::new(0, 0, usable_width, usable_height);
                let tag = TagIter::new(tags)
                    .filter_map(|tag| state.outputs.get(output).map(|output| &output.tags[tag]))
                    .next();
                if let Some(tag) = tag {
                    let layout = state.layouts.get(&tag.layout).unwrap_or(&Layout::Full);
                    LayoutGenerator::new(view_count, rect, layout, &state.layouts)
                        .parameters(tag.params)
                        .iter(|rect| {
                            let Rect { x, y, w, h } = rect.pad(tag.padding);
                            proxy.push_view_dimensions(x as i32, y as i32, w, h, serial);
                        });
                    proxy.commit(tag.layout.clone(), serial);
                }
            }
            river_layout_v3::Event::NamespaceInUse => {
                eprintln!("Namespace in use!");
            }
            river_layout_v3::Event::UserCommand { command } => {
                let output = state.outputs.get_mut(output).unwrap();
                for (tag, (command, value)) in TagIter::new(state.tags).zip(command.split_once(' '))
                {
                    let tag = &mut output.tags[tag];
                    match command {
                        "padding" => tag.padding = value.parse().unwrap_or(tag.padding),
                        "mod-padding" => tag.padding += value.parse::<i32>().unwrap_or_default(),
                        "main-count" => tag.params.0 = value.parse().ok(),
                        "mod-main-count" => {
                            tag.params.0 = Some(
                                (tag.params.0.unwrap_or_default() as i32)
                                    .saturating_add(value.parse().ok().unwrap_or_default())
                                    as u32,
                            );
                        }
                        "main-index" => tag.params.1 = value.parse().ok(),
                        "mod-main-index" => {
                            tag.params.1 = Some(
                                (tag.params.1.unwrap_or_default() as i32)
                                    .saturating_add(value.parse().ok().unwrap_or_default())
                                    as usize,
                            );
                        }
                        "main-ratio" => {
                            tag.params.2 = value.parse::<f64>().ok().map(|f| f.clamp(0., 1.))
                        }
                        "mod-main-ratio" => {
                            tag.params.2 = Some(
                                tag.params.2.unwrap_or_default()
                                    + value.parse::<f64>().ok().unwrap_or_default(),
                            )
                            .map(|f| f.clamp(0., 1.));
                        }
                        "layout" => {
                            tag.layout.replace_range(.., value);
                        }
                        "reload" => return state.load_layouts(),

                        "path" => {
                            state.path = value.into();
                            return state.load_layouts();
                        }
                        _ => {}
                    }
                }
            }
            river_layout_v3::Event::UserCommandTags { tags } => {
                state.tags = tags;
            }
        }
    }
}

impl Dispatch<RiverLayoutManagerV3, ()> for LayoutManager {
    fn event(
        _: &mut Self,
        _: &RiverLayoutManagerV3,
        _: <RiverLayoutManagerV3 as wayland_client::Proxy>::Event,
        _: &(),
        _: &wayland_client::Connection,
        _: &wayland_client::QueueHandle<Self>,
    ) {
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tag_test() {
        let iter = TagIter::new(1 | 8);

        let tags = iter.collect::<Vec<_>>();

        assert_eq!(&tags, &[0, 3])
    }
}
