use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use std::io::{Cursor, Read, Write};
use crate::util::pos::BlockPos;

pub const PROTOCOL_VERSION: u32 = 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PacketType {
    C2SMetadataRequest = 0,
    S2CMetadata = 1,
    C2SUnregisterReply = 2,
    S2CEntityNbtResponseSimple = 3,
    C2SSchematicChunk = 4,
    S2CSchematicChunk = 5,
}

impl PacketType {
    pub fn from_id(id: u8) -> Option<Self> {
        match id {
            0 => Some(PacketType::C2SMetadataRequest),
            1 => Some(PacketType::S2CMetadata),
            2 => Some(PacketType::C2SUnregisterReply),
            3 => Some(PacketType::S2CEntityNbtResponseSimple),
            4 => Some(PacketType::C2SSchematicChunk),
            5 => Some(PacketType::S2CSchematicChunk),
            _ => None,
        }
    }

    pub fn id(&self) -> u8 {
        *self as u8
    }
}

#[derive(Debug, Clone)]
pub struct ServuxLitematicaPacket {
    pub packet_type: PacketType,
    pub entity_id: i32,
    pub pos: BlockPos,
    pub payload: Vec<u8>,
}

impl ServuxLitematicaPacket {
    pub fn metadata_request(payload: Vec<u8>) -> Self {
        Self {
            packet_type: PacketType::C2SMetadataRequest,
            entity_id: -1,
            pos: BlockPos::ZERO,
            payload,
        }
    }

    pub fn metadata_response(payload: Vec<u8>) -> Self {
        Self {
            packet_type: PacketType::S2CMetadata,
            entity_id: -1,
            pos: BlockPos::ZERO,
            payload,
        }
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.write_u8(self.packet_type.id()).unwrap();
        buf.write_u32::<BigEndian>(PROTOCOL_VERSION).unwrap();
        buf.write_i32::<BigEndian>(self.entity_id).unwrap();
        buf.write_i32::<BigEndian>(self.pos.x).unwrap();
        buf.write_i32::<BigEndian>(self.pos.y).unwrap();
        buf.write_i32::<BigEndian>(self.pos.z).unwrap();
        buf.write_u32::<BigEndian>(self.payload.len() as u32).unwrap();
        buf.write_all(&self.payload).unwrap();
        buf
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, Box<dyn std::error::Error>> {
        let mut cursor = Cursor::new(bytes);
        let type_id = cursor.read_u8()?;
        let packet_type = PacketType::from_id(type_id)
            .ok_or_else(|| format!("Unknown packet type {}", type_id))?;

        let _proto = cursor.read_u32::<BigEndian>()?;
        let entity_id = cursor.read_i32::<BigEndian>()?;
        let x = cursor.read_i32::<BigEndian>()?;
        let y = cursor.read_i32::<BigEndian>()?;
        let z = cursor.read_i32::<BigEndian>()?;
        let payload_len = cursor.read_u32::<BigEndian>()? as usize;

        let mut payload = vec![0u8; payload_len];
        cursor.read_exact(&mut payload)?;

        Ok(Self {
            packet_type,
            entity_id,
            pos: BlockPos::new(x, y, z),
            payload,
        })
    }
}
