pub mod lexer;

pub use lexer::*;
use crate::client::*;
use crate::layout::*;

pub fn format<'s>(string: &'s str) -> (&'s str, &'s str) {
    let exp = lexer::Expression::new(string.as_ref()).split_ounce(' ');
    exp.drop()
}

// Handles string events
pub fn main<'s>(output_handle: &mut Output, name: &'s str, value: &'s str) {
    let mut command = value.split_whitespace();
    match name {
        "dimension" => {
            let mut arguments = value.split_whitespace();
            output_handle.dimension = {
                output_handle.resize = true;
                Area {
                    x: arguments.next().unwrap_or("0").parse::<u32>().unwrap(),
                    y: arguments.next().unwrap_or("0").parse::<u32>().unwrap(),
                    w: arguments.next().unwrap_or("500").parse::<u32>().unwrap(),
                    h: arguments.next().unwrap_or("500").parse::<u32>().unwrap(),
                }
            }
        }
        "resize" => {
            output_handle.resize = if let Ok(ans) = value.parse::<bool>() {
                ans
            } else {
                false
            }
        }
        "smart_padding" => {
            if let Ok(ans) = command.next().unwrap().parse::<bool>() {
                output_handle.smart_padding = ans;
            }
        }
        "clear" => {
            for arg in command {
                match arg {
                    "all" => output_handle.tags = Default::default(),
                    "focused" => output_handle.tags[output_handle.focused] = None,
                    _ => match arg.parse::<usize>() {
                        Ok(int) => {
                            if int > 0 && int < 33 {
                                output_handle.tags[int - 1] = None
                            }
                        }
                        Err(_) => {}
                    },
                }
            }
        }
        _ => {
            let tags: Result<std::ops::Range<usize>, ()> = match name.as_ref() {
                "focused" => Ok(output_handle.focused..output_handle.focused + 1),
                "all" => Ok(0..32),
                _ => match name.parse::<usize>() {
                    Ok(int) => Ok(int - 1..int),
                    Err(_) => Err(()),
                },
            };
            let mut main_index = 0;
            let mut main_amount = 1;
            let mut main_factor = 0.6;
            let mut main_layout = lexer::layout(&value);
            if let Layout::Assisted {
                layout,
                amount,
                factor,
                index,
            } = main_layout
            {
                main_layout = *layout;
                main_amount = amount;
                main_factor = factor;
                main_index = index;
            }
            if let Ok(tags) = tags {
                for i in tags {
                    let tag = output_handle.tags[i].as_mut();
                    match tag {
                        Some(tag) => {
                            tag.layout = main_layout.clone();
                            tag.parameters.main_index = main_index;
                            tag.parameters.main_amount = main_amount;
                            tag.parameters.main_factor = main_factor;
                        }
                        None => {
                            output_handle.tags[i] = Some({
                                Tag {
                                    layout: main_layout.clone(),
                                    parameters: {
                                        Parameters {
                                            main_index,
                                            main_amount,
                                            main_factor,
                                        }
                                    },
                                }
                            })
                        }
                    }
                }
            }
        }
    }
}
