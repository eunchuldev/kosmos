#![cfg_attr(target_arch = "spirv", no_std)]

use spirv_std::{glam::UVec3, spirv};
use kosmos_tile_shared::{TileConstants, Tile, Terrian};


// LocalSize/numthreads of (x = 64, y = 64, z = 64)
#[spirv(compute(threads(4, 4, 4)))]
pub fn tick(
    #[spirv(global_invocation_id)] id: UVec3,
    #[spirv(push_constant)] c: &TileConstants,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 0)] input: &[Tile],
    #[spirv(storage_buffer, descriptor_set = 0, binding = 1)] output: &mut [Tile],
) {
    if id.x == 0 || id.y == 0 || id.z == 0 || id.x == c.width-1 || id.y == c.height-1 || id.z == c.depth - 1 {
        return
    }
    let dir = match c.frame_number % 3 {
        0 => UVec3::new(1, 0, 0),
        1 => UVec3::new(0, 1, 0),
        2 => UVec3::new(0, 0, 1),
        _ => unreachable!(),
    };
    let idx_vec = UVec3::new(c.width*c.height, c.height, 1);
    let index = id.dot(idx_vec) as usize;
    let mut net_gravity = 0;
    for i in 0..2 {
        let neighbor_id = id + dir * (i*2 - 1);
        let index = neighbor_id.dot(idx_vec);
        net_gravity += input[index as usize].gravity;
    }
    output[index] = input[index];
    output[index].gravity = net_gravity / 2;

    /*match tiles[index].terrian {
        Terrian::DeepWater => {
            if tiles[index + ]
            tiles[index] = tiles[index];
        }
        Terrian::SwallowWater => {
        }
        _ => (),
    };*/
}


#[spirv(compute(threads(64)))]
pub fn sample(
    #[spirv(global_invocation_id)] id: UVec3,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 0)] tiles: &mut [u32],
) {
    let index = id.x as usize;
    tiles[index] = tiles[index];
}
