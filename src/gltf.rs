use std::path::Path;

use gltf::{buffer::Data as BufferData, image::Data as ImageData, Document, Semantic};

#[derive(Default)]
pub struct Mesh {
    pub positions: Vec<u8>,
    pub indices: Vec<u8>,

    pub normals: Vec<u8>,
    pub texture_coordinates: Vec<u8>,
}

pub struct GltfLoader {}

impl GltfLoader {
    pub fn load<P: AsRef<Path>>(path: P) -> Vec<Mesh> {
        let (schema, buffers, _): (Document, Vec<BufferData>, Vec<ImageData>) =
            gltf::import(path).unwrap();

        let mut meshes = Vec::new();

        for mesh in schema.meshes() {
            println!("mesh: {}", mesh.name().unwrap_or("none"));
            for primitive in mesh.primitives() {
                let mut mesh = Mesh::default();

                println!("new primitive");

                for (attribute_type, accessor) in primitive.attributes() {
                    println!(
                        "\tattribute: {:?} accessor component size: {:?}",
                        attribute_type,
                        accessor.size()
                    );

                    match attribute_type {
                        Semantic::Positions => {
                            mesh.positions =
                                GltfLoader::get_accessor_data(&schema, accessor.index(), &buffers);
                        }
                        Semantic::Normals => {
                            mesh.normals =
                                GltfLoader::get_accessor_data(&schema, accessor.index(), &buffers);
                        }
                        Semantic::Tangents => {}
                        Semantic::Colors(_) => {}
                        Semantic::TexCoords(_) => {
                            mesh.texture_coordinates =
                                GltfLoader::get_accessor_data(&schema, accessor.index(), &buffers);
                        }
                        Semantic::Joints(_) => {}
                        Semantic::Weights(_) => {}
                    }
                }

                println!(
                    "\tindices component size: {:?}",
                    primitive.indices().unwrap().size()
                );

                mesh.indices = GltfLoader::get_accessor_data(
                    &schema,
                    primitive
                        .indices()
                        .expect("Primitive doesn't have index buffer!")
                        .index(),
                    &buffers,
                );

                meshes.push(mesh);
            }
        }

        meshes
    }

    fn get_accessor_data(
        schema: &Document,
        accessor_index: usize,
        buffers: &Vec<BufferData>,
    ) -> Vec<u8> {
        let accessor = schema.accessors().nth(accessor_index).unwrap();
        let buffer_view = accessor.view().expect("Sparse accessors not supported!"); //TODO sparse accessors
        let buffer = buffer_view.buffer();

        let buffer_all_data = buffers.get(buffer.index()).unwrap();
        let buffer_view_data = {
            //If tightly packed
            if buffer_view.stride().is_none() {
                buffer_all_data[buffer_view.offset()..buffer_view.offset() + buffer_view.length()]
                    .to_vec()
            }
            //TODO sparse view data
            else {
                panic!("Sparse view data not supported!");
            }
        };
        let accessor_data = buffer_view_data
            [accessor.offset()..accessor.offset() + accessor.size() * accessor.count()]
            .to_vec();

        accessor_data
    }
}
