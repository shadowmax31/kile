use super::client::*;

#[derive(Clone, Debug)]
pub enum Layout {
    Full,
    Deck,
    Vertical,
    Horizontal,
    Recursive {
        outer: Box<Layout>,
        inner: Vec<Layout>,
    },
    Assisted {
        layout: Box<Layout>,
        amount: u32,
        index: u32,
        factor: f64,
    },
}

impl Area {
    pub fn apply_padding(&mut self, padding: i32) {
        if 2 * padding < self.h as i32 && 2 * padding < self.w as i32 {
            self.x = ((self.x as i32) + padding) as u32;
            self.y = ((self.y as i32) + padding) as u32;
            self.w = ((self.w as i32) - 2 * padding) as u32;
            self.h = ((self.h as i32) - 2 * padding) as u32;
        }
    }
    pub fn generate(
        self,
        parameters: &Parameters,
        mut client_count: u32,
        layout: &Layout,
        list: &mut Vec<Area>,
        parent: bool,
        factor: bool,
    ) {
        let mut area = self;
        let master = parent && factor && client_count > 1 && parameters.main_index < client_count;

        match layout {
            Layout::Full => {
                for _i in 0..client_count {
                    list.push(area);
                }
            }
            Layout::Deck => {
                let yoffset = ((self.h as f64 * 0.1) / (client_count as f64 - 1.0)).floor() as u32;
                let xoffset = ((self.w as f64 * 0.1) / (client_count as f64 - 1.0)).floor() as u32;
                for _i in 0..client_count {
                    area.w = self.w - (xoffset * (client_count - 1));
                    area.h = self.h - (yoffset * (client_count - 1));
                    list.push(area);
                    area.x += xoffset;
                    area.y += yoffset;
                }
            }
            Layout::Horizontal => {
                let reste = area.h % client_count;
                let mut slave_height = area.h;
                let main_height = if master {
                    ((area.h as f64) * parameters.main_factor) as u32
                } else {
                    0
                };
                slave_height -= main_height;
                for i in 0..client_count {
                    area.h = if master && i == parameters.main_index {
                        main_height
                    } else if master {
                        slave_height / (client_count - 1)
                    } else {
                        slave_height / client_count
                    };
                    if i == 0 {
                        area.h += reste;
                    }

                    list.push(area);
                    area.y += area.h;
                }
            }
            Layout::Vertical => {
                let reste = area.w % client_count;
                let mut slave_width = area.w;
                let main_width = if master {
                    ((area.w as f64) * parameters.main_factor) as u32
                } else {
                    0
                };
                slave_width -= main_width;
                for i in 0..client_count {
                    area.w = if master && i == parameters.main_index {
                        main_width
                    } else if master {
                        slave_width / (client_count - 1)
                    } else {
                        slave_width / client_count
                    };
                    if i == 0 {
                        area.w += reste;
                    }

                    list.push(area);
                    area.x += area.w;
                }
            }
            Layout::Recursive { outer, inner } => {
                let mut frame = Vec::new();
                let frames_available = inner.len() as u32;
                let mut frame_amount = {
                    let main = parameters.main_amount >= 1
                        && frames_available > 1
                        && parameters.main_index < frames_available
                        && client_count > parameters.main_amount;
                    if parameters.main_amount >= client_count {
                        1
                    } else if main && client_count - parameters.main_amount < frames_available {
                        1 + client_count - parameters.main_amount
                    } else if client_count > frames_available || main {
                        frames_available
                    } else {
                        client_count
                    }
                };
                area.generate(parameters, frame_amount, &*outer, &mut frame, true, factor);
                if parent && parameters.main_amount > 0 && parameters.main_index < frame_amount {
                    frame_amount -= 1;
                    client_count -= parameters.main_amount;
                    frame.remove(parameters.main_index as usize).generate(
                        parameters,
                        parameters.main_amount,
                        &inner[parameters.main_index as usize],
                        list,
                        false,
                        false,
                    );
                }
                for (mut i, rect) in frame.iter_mut().enumerate() {
                    let mut count = client_count / frame_amount;
                    if client_count % frame_amount != 0 && i as u32 != frame_amount {
                        client_count -= 1;
                        count += 1;
                    }
                    if master && parameters.main_amount > 0 && i >= parameters.main_index as usize {
                        i += 1
                    }
                    rect.generate(parameters, count, &inner[i], list, false, false)
                }
            }
            Layout::Assisted {
                layout,
                amount,
                index,
                factor,
            } => {
                let parameters = {
                    Parameters {
                        main_amount: *amount,
                        main_index: *index,
                        main_factor: *factor,
                    }
                };
                area.generate(&parameters, client_count, &*layout, list, true, true);
            }
        }
    }
}
