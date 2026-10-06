use super::{CurrentGameError, CurrentSgfDocument, NodePath, SgfNode};
use app_model::CurrentGameErrorKind;

impl CurrentSgfDocument {
    /// Replaces an external source revision without treating move counts or equal boards as identity.
    /// Only unchanged ancestry at the same path can retain locally changed comments and analysis.
    pub fn reconcile_external_source(
        &self,
        previous_source: &Self,
        incoming: &Self,
        selected: &NodePath,
        jump_to_last: bool,
    ) -> Result<(Self, NodePath), CurrentGameError> {
        // An invalid live cursor is an error, unlike a valid cursor removed by the new source.
        let current_root = self.root()?;
        let mut selected_node = current_root;
        for &index in &selected.indices {
            selected_node = selected_node
                .children
                .get(index as usize)
                .ok_or_else(|| CurrentGameError {
                    kind: CurrentGameErrorKind::InvalidNodePath,
                    message: "invalid node path".into(),
                })?;
        }
        let previous_root = previous_source.root()?;
        let incoming_root = incoming.root()?;
        let root_matches = same_root_semantics(current_root, previous_root)
            && same_root_semantics(previous_root, incoming_root)
            && same_position_properties(current_root, previous_root)
            && same_position_properties(previous_root, incoming_root);
        let relocated = if jump_to_last || !root_matches {
            incoming.default_selected_path()
        } else {
            matching_selected_ancestor(current_root, previous_root, incoming_root, selected)
        };
        let mut merged = incoming.clone();
        if root_matches {
            reconcile_matching_nodes(
                current_root,
                previous_root,
                merged.document.root.as_mut().expect("incoming root checked"),
            );
        }
        Ok((merged, relocated))
    }
}

fn same_root_semantics(left: &SgfNode, right: &SgfNode) -> bool {
    // Exact property equality is deliberately conservative about different encodings of a value.
    // Metadata order is irrelevant, but absence, duplicate properties and all values participate.
    ["SZ", "KM", "RU", "HA"]
        .into_iter()
        .all(|key| same_property(left, right, key))
}

fn same_position_properties(left: &SgfNode, right: &SgfNode) -> bool {
    let position_property = |key: &str| matches!(key, "B" | "W" | "AB" | "AW" | "AE" | "PL");
    left.properties
        .iter()
        .filter(|property| position_property(&property.key))
        .eq(right
            .properties
            .iter()
            .filter(|property| position_property(&property.key)))
}

fn same_property(left: &SgfNode, right: &SgfNode, key: &str) -> bool {
    left.properties
        .iter()
        .filter(|property| property.key == key)
        .eq(right.properties.iter().filter(|property| property.key == key))
}

fn matching_selected_ancestor(
    mut current: &SgfNode,
    mut previous: &SgfNode,
    mut incoming: &SgfNode,
    selected: &NodePath,
) -> NodePath {
    let mut indices = Vec::with_capacity(selected.indices.len());
    for &index in &selected.indices {
        let (Some(current_child), Some(previous_child), Some(incoming_child)) = (
            current.children.get(index as usize),
            previous.children.get(index as usize),
            incoming.children.get(index as usize),
        ) else {
            break;
        };
        if !same_position_properties(current_child, previous_child)
            || !same_position_properties(previous_child, incoming_child)
        {
            break;
        }
        indices.push(index);
        current = current_child;
        previous = previous_child;
        incoming = incoming_child;
    }
    NodePath { indices }
}

fn reconcile_matching_nodes(current: &SgfNode, previous: &SgfNode, incoming: &mut SgfNode) {
    for key in ["C", "LZ", "LZ2", "LZOP", "LZOP2"] {
        if !same_property(current, previous, key)
            && (key == "C" || current.properties.iter().any(|property| property.key == key))
        {
            incoming.properties.retain(|property| property.key != key);
            incoming.properties.extend(
                current
                    .properties
                    .iter()
                    .filter(|property| property.key == key)
                    .cloned(),
            );
        }
    }
    for ((current_child, previous_child), incoming_child) in current
        .children
        .iter()
        .zip(&previous.children)
        .zip(&mut incoming.children)
    {
        if same_position_properties(current_child, previous_child)
            && same_position_properties(previous_child, incoming_child)
        {
            reconcile_matching_nodes(current_child, previous_child, incoming_child);
        }
    }
}
