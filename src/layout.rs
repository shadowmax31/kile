// Layouts for river

#[derive(Copy, Clone)]

pub struct Window{
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32
}

pub fn stack(mut node_tree:Vec<Window>, client_count:u32, master_count:u32, master_width_factor:f32, screen_width:u32, screen_height:u32) -> Vec<Window> {

    let master_width: u32=screen_width*((master_width_factor * 100.0) as u32)/100;

    if client_count<2 {
        let fullscreen=Window{
            x: 0,
            y: 0,
            width: screen_width,
            height: screen_height,
        };

        node_tree.push(fullscreen);
        return node_tree;
    }

    let master=Window{
        x: 0,
        y: 0,
        width: master_width,
        height: screen_height,
    };

    let mut slave=Window{
        x: master_width,
        y: 0,
        width: screen_width-master_width,
        height: screen_height,
    };

    for i in 1..client_count+1 {
        if i == master_count {
            node_tree.push(master);
        }
        else if (master_count < 1 || master_count > client_count) && i == 1 {
            node_tree.push(master);
        }
        else {
            node_tree.push(slave);
            slave.height-=30;
            slave.y+=30;
        }
    }

    node_tree
}

pub fn hive(mut node_tree:Vec<Window>, client_count:u32, master_count:u32, master_width_factor:f32, screen_width:u32, screen_height:u32) -> Vec<Window> {

    let master_width: u32=screen_width*((master_width_factor * 100.0) as u32)/150;

    let branch_count: u32=(client_count-1)/2;

    if client_count<3 {
        node_tree=grid(node_tree, client_count, master_count, master_width_factor, screen_width, screen_height);
        return node_tree
    } else {

        let mut slave=Window{
            x: 0,
            y: 0,
            width: (screen_width-master_width)/2,
            height: screen_height/branch_count,
        };

        let master=Window{
            x: (screen_width-master_width)/2,
            y: 0,
            width: master_width,
            height: screen_height,
        };

        let left_client_height:u32=screen_height/branch_count;
        let right_client_height:u32=screen_height/(client_count-branch_count-1);

        let mut left_count:u32=0;
        let mut right_count:u32=0;

        for i in 1..client_count+1 {
            if i==master_count {
                node_tree.push(master);
            }
            else if (master_count < 1 || master_count > client_count) && i == 1 {
                node_tree.push(master);
            }
            else {
                if left_count<branch_count {
                    slave.height= if left_count < branch_count-1 {
                        left_client_height
                    } else { left_client_height+(screen_height%left_client_height) };

                    node_tree.push(slave);

                    slave.y+=slave.height;
                    left_count+=1;
                }
                else {
                    slave.x=master_width+((screen_width-master_width)/2);

                    slave.height= if right_count!=1 {
                        right_client_height
                    } else { right_client_height+(screen_height%right_client_height) };

                    if right_count==0 {
                       slave.y=0;
                    } else {
                       slave.y+=slave.height;
                    };

                    node_tree.push(slave);
                    right_count+=1;
                }
            }
        }
    }

    node_tree

}

pub fn grid(mut node_tree:Vec<Window>, client_count:u32, master_count:u32, master_width_factor:f32, screen_width:u32, screen_height:u32) -> Vec<Window> {

    let views:u32=(client_count as f32).sqrt().ceil() as u32;

    let master_view:u32= if views*views != client_count {
        1
    } else { 0 };

    let mut master_view_width:u32=screen_width*((master_width_factor * 100.0) as u32)/100;

    if 50 > master_view_width || master_view_width > screen_width-50 {
        master_view_width=screen_width/views;
    }

    let slave_view_width:u32=(screen_width-master_view_width)/(views-1);

    let mut window=Window{
        x:0,
        y:0,
        width: 0,
        height:0,
    };

    let master_clients:u32=client_count-((views-master_view)*(views-master_view));

    for i in 0..views {

        if i < views-master_view {
            if i==0 {
                window.width=master_view_width;
            } else {
                window.width=slave_view_width;
            }

            for w in 0..views-master_view {
                window.height=screen_height/(views-master_view);
                window.y=window.height*w;

                if window.height+window.y > screen_height {
                    window.height-=screen_height%window.height;
                } else if window.height+window.y < screen_height {
                    window.height+=screen_height%window.height;
                }
                node_tree.push(window);
            }

            window.x+=window.width;

        } else {

            window.width=slave_view_width;

            for w in 0..master_clients {
                window.height=screen_height/master_clients;
                window.y=window.height*w;

                if window.height+window.y > screen_height {
                    window.height-=screen_height%window.height;
                } else if window.height+window.y < screen_height {
                    window.height+=screen_height%window.height;
                }
                node_tree.push(window);
            }

            window.x+=window.width;

        }

    }

    node_tree

}

pub fn left(mut node_tree:Vec<Window>, client_count:u32, master_count:u32, master_width_factor:f32, screen_width:u32, screen_height:u32) -> Vec<Window> {

    let mut window=Window{
        x:0,
        y:0,
        width:0,
        height:0,
    };

    let master_count:u32=if master_count > client_count {
        client_count
    } else { master_count };

    let slave_count:u32=client_count-master_count;

    let master_width: u32= if client_count > master_count {
        screen_width*((master_width_factor * 100.0) as u32)/100
    } else { screen_width };

    let slave_width:u32= if master_count >= 1 {
        screen_width-master_width
    } else { screen_width };

    for i in 0..client_count {

        if i < master_count  {
            window.width=master_width;
            window.height= if i != 0 {
                screen_height/master_count
            } else { (screen_height/master_count)+(screen_height%(screen_height/master_count)) };
            window.y=window.height*i;
        } else {
            if master_count!=0 {
                window.x=master_width;
            }
             if i != master_count {
                window.y+=window.height;
                window.height=screen_height/slave_count;
            } else {
                window.height=(screen_height/slave_count)+(screen_height%(screen_height/slave_count));
                window.y=0;
            }
            window.width=slave_width;
        }

        node_tree.push(window);

    }

    node_tree

}