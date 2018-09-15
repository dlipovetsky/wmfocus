use i3ipc::reply::{Node, NodeType, Workspace};
use i3ipc::I3Connection;
use DesktopWindow;

/// Find first `Node` that fulfills a given criterion.
fn find_first_node_with_attr<F>(start_node: &Node, predicate: F) -> Option<&Node>
where
    F: Fn(&Node) -> bool,
{
    let mut nodes_to_explore: Vec<&Node> = start_node.nodes.iter().collect();
    while !nodes_to_explore.is_empty() {
        let mut next_vec = vec![];
        for node in &nodes_to_explore {
            if predicate(node) {
                return Some(node);
            }
            next_vec.extend(node.nodes.iter());
        }
        nodes_to_explore = next_vec;
    }
    None
}

/// Return a list of all `DesktopWindow`s for the given `Workspace`.
fn crawl_windows(root_node: &Node, workspace: &Workspace) -> Vec<DesktopWindow> {
    let workspace_node = find_first_node_with_attr(&root_node, |x| {
        x.name == Some(workspace.name.clone()) && if let NodeType::Workspace = x.nodetype {
            true
        } else {
            false
        }
    }).expect("Couldn't find the Workspace node");

    let mut nodes_to_explore: Vec<&Node> = workspace_node.nodes.iter().collect();
    nodes_to_explore.extend(workspace_node.floating_nodes.iter());
    let mut windows = vec![];
    while !nodes_to_explore.is_empty() {
        let mut next_vec = vec![];
        for node in &nodes_to_explore {
            next_vec.extend(node.nodes.iter());
            next_vec.extend(node.floating_nodes.iter());
            if node.window.is_some() {
                let window = DesktopWindow {
                    id: node.id,
                    title: node.name.clone().unwrap_or_default(),
                    pos: ((node.rect.0), (node.rect.1 - node.deco_rect.3)),
                    size: ((node.rect.2), (node.rect.3 + node.deco_rect.3)),
                };
                windows.push(window);
            }
        }
        nodes_to_explore = next_vec;
    }
    windows
}

/// Return a list of all windows.
pub fn get_windows() -> Vec<DesktopWindow> {
    // Establish a connection to i3 over a unix socket
    let mut connection = I3Connection::connect().expect("Couldn't acquire i3 connection");
    let workspaces = connection
        .get_workspaces()
        .expect("Problem communicating with i3")
        .workspaces;
    let visible_workspaces = workspaces.iter().filter(|w| w.visible);
    let root_node = connection.get_tree().expect("Uh");
    let mut windows = vec![];
    for workspace in visible_workspaces {
        windows.extend(crawl_windows(&root_node, &workspace));
    }
    windows
}

/// Focus a specific `window`.
pub fn focus_window(window: &DesktopWindow) {
    let mut connection = I3Connection::connect().expect("Couldn't acquire i3 connection");
    let command_str = format!("[con_id=\"{}\"] focus", window.id);
    let command = connection
        .run_command(&command_str)
        .expect("Couldn't communicate with i3");
    info!("Sending to i3: {:?}", command);
}
