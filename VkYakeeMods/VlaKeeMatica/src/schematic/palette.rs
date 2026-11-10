use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BlockState {
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "Properties", default, skip_serializing_if = "BTreeMap::is_empty")]
    pub properties: BTreeMap<String, String>,
}

impl BlockState {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            properties: BTreeMap::new(),
        }
    }

    pub fn air() -> Self {
        Self::new("minecraft:air")
    }

    pub fn is_air(&self) -> bool {
        self.name == "minecraft:air" || self.name == "minecraft:cave_air" || self.name == "minecraft:void_air"
    }

    pub fn with_property(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.properties.insert(key.into(), value.into());
        self
    }

    pub fn get_property(&self, key: &str) -> Option<&str> {
        self.properties.get(key).map(|s| s.as_str())
    }

    pub fn to_string_repr(&self) -> String {
        if self.properties.is_empty() {
            self.name.clone()
        } else {
            let props: Vec<_> = self.properties.iter().map(|(k, v)| format!("{}={}", k, v)).collect();
            format!("{}[{}]", self.name, props.join(","))
        }
    }

    pub fn from_string_repr(s: &str) -> Self {
        if let Some(bracket_idx) = s.find('[') {
            let name = &s[..bracket_idx];
            let end_idx = s.rfind(']').unwrap_or(s.len());
            let prop_str = &s[bracket_idx + 1..end_idx];
            let mut properties = BTreeMap::new();
            for part in prop_str.split(',') {
                if let Some((k, v)) = part.split_once('=') {
                    properties.insert(k.trim().to_string(), v.trim().to_string());
                }
            }
            Self {
                name: name.to_string(),
                properties,
            }
        } else {
            Self::new(s)
        }
    }
}

impl Default for BlockState {
    fn default() -> Self {
        Self::air()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlockStatePalette {
    palette: Vec<BlockState>,
    #[serde(skip)]
    state_to_id: HashMap<BlockState, u32>,
}

impl Default for BlockStatePalette {
    fn default() -> Self {
        Self::new()
    }
}

impl BlockStatePalette {
    pub fn new() -> Self {
        let air = BlockState::air();
        let mut map = HashMap::new();
        map.insert(air.clone(), 0);
        Self {
            palette: vec![air],
            state_to_id: map,
        }
    }

    pub fn from_states(states: Vec<BlockState>) -> Self {
        let mut map = HashMap::new();
        for (i, state) in states.iter().enumerate() {
            map.insert(state.clone(), i as u32);
        }
        Self {
            palette: states,
            state_to_id: map,
        }
    }

    pub fn rebuild_map(&mut self) {
        self.state_to_id.clear();
        for (i, state) in self.palette.iter().enumerate() {
            self.state_to_id.insert(state.clone(), i as u32);
        }
    }

    pub fn id_for(&mut self, state: &BlockState) -> u32 {
        if let Some(&id) = self.state_to_id.get(state) {
            return id;
        }

        let id = self.palette.len() as u32;
        self.palette.push(state.clone());
        self.state_to_id.insert(state.clone(), id);
        id
    }

    pub fn get_state(&self, id: u32) -> Option<&BlockState> {
        self.palette.get(id as usize)
    }

    pub fn get_state_or_air(&self, id: u32) -> BlockState {
        self.get_state(id).cloned().unwrap_or_else(BlockState::air)
    }

    pub fn len(&self) -> usize {
        self.palette.len()
    }

    pub fn is_empty(&self) -> bool {
        self.palette.is_empty()
    }

    pub fn as_slice(&self) -> &[BlockState] {
        &self.palette
    }

    pub fn into_vec(self) -> Vec<BlockState> {
        self.palette
    }
}
