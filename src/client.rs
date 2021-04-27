use super::parser;
use super::options::{Layout, Options};
use crate::wayland::{
    river_layout_v2::river_layout_manager_v2::RiverLayoutManagerV2,
    river_layout_v2::river_layout_v2::Event,
};
use wayland_client::protocol::wl_output::WlOutput;
use wayland_client::{DispatchData, Main};

pub struct Context {
    pub running: bool,
    pub namespace: String,
    pub outputs: Vec<Output>,
    pub layout_manager: Option<Main<RiverLayoutManagerV2>>,
}

pub struct Output {
    pub options: Options,
    pub focused: usize,
    pub configured: bool,
    pub default: Tag,
    pub output: Option<WlOutput>,
    pub tags: [Option<Tag>; 32],
}

#[derive(Clone)]
pub struct Tag {
    pub outer: Layout,
    pub main_index: u32,
    pub main_amount: u32,
    pub main_factor: f64,
    pub inner: Vec<Layout>,
    pub rule: Rule,
    pub frame: Option<Frame>,
}

#[derive(Clone, Debug)]
pub struct Window {
    pub area: Option<Area>,
    pub app_id: String,
    pub tags: u32,
}

#[derive(Clone)]
pub struct Frame {
    pub layout: Layout,
    pub area: Area,
    pub list: Vec<Window>,
}

#[derive(Copy, Clone, Debug)]
pub struct Area {
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
}

#[derive(Clone, Debug)]
pub enum Rule {
    AppId ( String ),
    Tag ( u32 ),
    None,
}

pub trait Rectangle {
    fn apply_padding(&mut self, padding: u32);
    fn push_dimensions(&mut self, options: &Options);
}

impl Context {
    pub fn new() -> Context {
        return {
            Context {
                running: false,
                namespace: String::from("kile"),
                layout_manager: None,
                outputs: Vec::new(),
            }
        };
    }
    pub fn init(&mut self, monitor_index: usize) {
        for output in &mut self.outputs {
            if !output.configured {
                output.get_layout(self.layout_manager.as_ref(), self.namespace.clone());
            } else {
                output.destroy();
            }
        }
    }
}

impl Output {
    pub fn new(output: WlOutput) -> Output {
        {
            Output {
                configured: false,
                focused: 0,
                options: Options::new(),
                default: Tag::new(),
                output: Some(output),
                tags: Default::default(),
            }
        }
    }
    pub fn destroy(&self) {
        (self.options.layout).as_ref().unwrap().destroy();
    }
    pub fn update(&mut self) {
        if self.options.view_amount > 0 {
            // self.options.debug();
            let focused = self.tags[self.focused].as_mut();
            self.options.rearrange();
            match focused {
                Some(tag) => tag.update(&mut self.options),
                None => self.default.update(&mut self.options),
            }
        }
    }
    pub fn get_layout(&mut self, layout_manager: Option<&Main<RiverLayoutManagerV2>>, namespace: String) {
        self.options.layout = Some(
            layout_manager
                .expect("Compositor doesn't implement river_layout_v2")
                .get_layout(self.output.as_ref().unwrap(), namespace),
        );
        self.options.layout.as_ref().unwrap().quick_assign(
            move |_, event, mut output: DispatchData| { 
                let output_handle = output.get::<Output>().unwrap();
                match event {
                    Event::LayoutDemand {
                        view_count,
                        usable_width,
                        usable_height,
                        serial,
                        mut tags,
                    } => {
                        output_handle.options.serial = serial;
                        output_handle.options.view_amount = view_count;
                        output_handle.options.usable_height = usable_height;
                        output_handle.options.usable_width = usable_width;
                        output_handle.focused = {
                            let mut i = 0;
                            while tags > 1 {
                                tags /= 2;
                                i += 1;
                            }
                            i as usize
                        };
                    }
                    Event::AdvertiseView {
                        tags,
                        app_id,
                        serial: _,
                    } => {
                        output_handle
                            .options
                            .windows
                            .push(Window {
                                app_id: app_id.unwrap(),
                                area: None,
                                tags: tags,
                            });
                    }
                    Event::NamespaceInUse => {
                        println!("Namespace already in use.");
                    }
                    Event::AdvertiseDone { serial } => {
                        output_handle.update()
                    }
                    Event::SetIntValue{ name , value } => match name.as_ref() {
                        "main_amount" | "main_index" => {
                            if let Some(tag) =
                                output_handle.tags[output_handle.focused].as_mut()
                            {
                                if value > 0 {
                                    match name.as_ref() {
                                        "main-amount" => tag.main_amount = value as u32,
                                        "main-index" => tag.main_index = value as u32,
                                        _ => {}
                                    }
                                }
                            }
                        }
                        "view_padding" => output_handle.options.view_padding = value as u32,
                        "outer_padding" => output_handle.options.outer_padding = value as u32,
                        "xoffset" => output_handle.options.xoffset = value,
                        "yoffset" => output_handle.options.yoffset = value,
                        _ => {}
                    },
                    Event::ModIntValue{ name , delta } => match name.as_ref() {
                        "main_amount" | "main_index" => {
                            if let Some(tag) =
                                output_handle.tags[output_handle.focused].as_mut()
                            {
                                match name.as_ref() {
                                    "main_amount" => tag.main_amount = ((tag.main_amount as i32) + delta) as u32,
                                    "main_index" => tag.main_index = ((tag.main_amount as i32) + delta) as u32,
                                    _ => {}
                                }
                            }
                        }
                        "view_padding" => output_handle.options.view_padding = ((output_handle.options.view_padding as i32) + delta) as u32,
                        "outer_padding" => output_handle.options.outer_padding = ((output_handle.options.outer_padding as i32) + delta) as u32,
                        "xoffset" => output_handle.options.xoffset += delta,
                        "yoffset" => output_handle.options.yoffset += delta,
                        _ => {}
                    },
                    Event::SetFixedValue{ name , value } => if name == "main_factor" { 
                        if let Some(tag) =
                            output_handle.tags[output_handle.focused].as_mut()
                        {
                            if value > 0.0 && value < 1.0 {
                                tag.main_factor = value
                            }
                        }
                    },
                    Event::ModFixedValue{ name , delta } => if name == "main_factor" { 
                        if let Some(tag) =
                            output_handle.tags[output_handle.focused].as_mut()
                        {
                            if delta <= tag.main_factor {
                                tag.main_factor += delta;
                            }
                        }
                    },
                    Event::SetStringValue{ name , value } => if name == "command" { parser::main(output_handle, value) },
                }
            }
        );
    }
}

impl Tag {
    pub fn new() -> Tag {
        {
            Tag {
                frame: None,
                main_index: 0,
                main_amount: 1,
                main_factor: 0.6,
                rule: Rule::None,
                outer: Layout::Full,
                inner: vec![Layout::Full],
            }
        }
    }
    fn push_views(&mut self, options: &Options) {
        let frame = self.frame.as_mut().unwrap();
        frame.focus_all(&self.rule, options.main_amount);
        for window in &mut frame.list {
            window.push_dimensions(options);
        }
        options.commit();
    }
    pub fn update(&mut self, options: &mut Options) {
        options.main_amount = self.main_amount;
        options.main_index = self.main_index;
        options.main_factor = self.main_factor;

        // Initialise a frame with the output dimension
        self.frame = Some(Frame::new(self.outer, options.get_output()));

        // Get a reference to the frame
        let outer_frame = self.frame.as_mut().unwrap();
        // The total amount of frame
        let frame_amount = options.frames_amount(self.inner.len() as u32);

        // Generate the outer layout
        outer_frame.generate(frame_amount, options, true, true);

        let main_amount = options.main_amount(frame_amount);
        let slave_amount;
        let mut reste = if main_amount > 0 {
            outer_frame.zoom(options.main_index as usize);
            slave_amount = (options.view_amount - main_amount) / (frame_amount - 1);
            (options.view_amount - main_amount) % (frame_amount - 1)
        } else {
            slave_amount = options.view_amount / frame_amount;
            options.view_amount % frame_amount
        };

        // Generate the inner layouts
        let mut windows = Vec::new();
        for (i, window) in outer_frame.list.iter().enumerate() {
            let mut frame = Frame::new(self.inner[i], window.area.unwrap());
            if i == 0 && main_amount != 0 {
                frame.generate(main_amount, options, false, false);
            } else {
                frame.generate(
                    if reste > 0 {
                        reste -= 1;
                        slave_amount + 1
                    } else {
                        slave_amount
                    },
                    options,
                    false,
                    false,
                );
            }
            windows.append(&mut frame.list);
        }
        outer_frame.list = windows;

        // Push views to the server
        self.push_views(options);
    }
}

impl Window {
    pub fn apply_padding(&mut self, padding: u32) {
        let mut area = self.area.unwrap();
        area.apply_padding(padding);
        self.area = Some(area);
    }
    pub fn push_dimensions(&mut self, options: &Options) {
        if !options.smart_padding || options.view_amount > 1 {
            self.apply_padding(options.view_padding);
        }
        options.push_dimensions(&self.area.unwrap());
    }
    fn compare(&self, rule: &Rule) -> bool {
        match rule {
            Rule::AppId ( string ) => string.eq(&self.app_id),
            Rule::Tag ( uint ) => self.tags == *uint,
            _ => false,
        }
    }
}

impl Area {
    pub fn apply_padding(&mut self, padding: u32) {
        if 2 * padding < self.h && 2 * padding < self.w {
            self.x += padding;
            self.y += padding;
            self.w -= 2 * padding;
            self.h -= 2 * padding;
        }
    }
}

impl Frame {
    pub fn new(layout: Layout, area: Area) -> Frame {
        {
            Frame {
                layout: layout,
                area: area,
                list: Vec::new(),
            }
        }
    }
    pub fn zoom(&mut self, index: usize) {
        if (index as usize) < self.list.len() {
            let main = self.list[index].area;
            for i in (0..index).rev() {
                self.list[i + 1].area = self.list[i].area;
            }
            self.list[0].area = main;
        }
    }
    pub fn focus(&mut self, index: usize, to: usize) {
        if (index as usize) < self.list.len() {
            let main = self.list[to].area;
            for i in to..index {
                self.list[i].area = self.list[i + 1].area;
            }
            self.list[index].area = main;
        }
    }
    pub fn focus_all(&mut self, rule: &Rule, main_amount: u32) {
        let mut i = self.list.len() - 1;
        let mut zoomed = 0;
        let mut to = 0;
        while to < i && self.list[to].compare(rule) {
            to += 1;
        }
        while i > 0 {
            let mut j = i;
            while j > to && zoomed < main_amount && self.list[i].compare(rule) {
                self.focus(i, to);
                j -= 1;
                zoomed += 1;
            }
            if i != j {
                i = j
            } else {
                i -= 1
            }
        }
    }
    fn insert_window(&mut self, area: Area, options: &mut Options, parent: bool) {
        let mut window = if !parent && options.windows.len() > 0 {
            options.windows.remove(0)
        } else {
            {
                Window {
                    app_id: String::new(),
                    area: None,
                    tags: 0,
                }
            }
        };
        window.area = Some(area);
        self.list.push(window);
    }
    pub fn generate(
        &mut self,
        client_count: u32,
        options: &mut Options,
        parent: bool,
        mut factor: bool,
    ) {
        let mut area = self.area;
        let mut slave_area = area;
        let mut main_area = area;

        match self.layout {
            Layout::Tab => {
                for _i in 0..client_count {
                    self.insert_window(area, options, parent);
                    area.h -= 50;
                    area.y += 50;
                }
            }
            Layout::Horizontal => {
                let reste = area.h % client_count;
                if factor {
                    main_area.h = if options.main_amount > 0
                        && client_count > 1
                        && options.main_amount < options.view_amount
                        && options.main_index < client_count
                    {
                        (area.h * (options.main_factor * 100.0) as u32) / (50 * client_count)
                    } else {
                        0
                    };
                    slave_area.h -= main_area.h;
                }
                for i in 0..client_count {
                    area.h = if factor && i == options.main_index && main_area.h > 0 {
                        main_area.h
                    } else if factor && main_area.h > 0 {
                        slave_area.h / (client_count - 1)
                    } else {
                        slave_area.h / client_count
                    };
                    if i == 0 {
                        area.h += reste;
                    }

                    self.insert_window(area, options, parent);
                    area.y += area.h;
                }
            }
            Layout::Vertical => {
                let reste = area.w % client_count;
                if factor {
                    main_area.w = if options.main_amount > 0
                        && client_count > 1
                        && options.main_amount < options.view_amount
                        && options.main_index < client_count
                    {
                        (area.w * (options.main_factor * 100.0) as u32) / (50 * client_count)
                    } else {
                        0
                    };
                    slave_area.w -= main_area.w;
                }
                for i in 0..client_count {
                    area.w = if factor && i == options.main_index && main_area.w > 0 {
                        main_area.w
                    } else if factor && main_area.w > 0 {
                        slave_area.w / (client_count - 1)
                    } else {
                        slave_area.w / client_count
                    };
                    if i == 0 {
                        area.w += reste;
                    }

                    self.insert_window(area, options, parent);
                    area.x += area.w;
                }
            }
            Layout::Recursive { modi } => {
                for i in 0..client_count {
                    self.layout = if (i + modi) % 2 == 0 {
                        Layout::Vertical
                    } else {
                        Layout::Horizontal
                    };
                    if i < client_count - 1 {
                        self.generate(2, options, true, factor);
                        let index = self.list.len() - 1;
                        self.area = self.list.remove(index).area.unwrap();
                        if !parent && options.windows.len() > 0 {
                            let mut window = options.windows.remove(0);
                            window.area = self.list[index - 1].area;
                            self.list[index - 1] = window;
                        }
                    } else {
                        self.generate(1, options, parent, factor);
                    }
                    factor = false;
                }
            }
            Layout::Full => {
                for _i in 0..client_count {
                    self.insert_window(area, options, parent);
                }
            }
        }
    }
}
