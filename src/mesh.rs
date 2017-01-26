use std::path::Path;
use tobj;
use cgmath::Point2;
use cgmath::Point3;
use cgmath::Vector3;
use primitive::triangle::Triangle;
use material::Material;

pub fn load_mesh(path: &Path) -> Vec<Triangle> {
    let obj_data = tobj::load_obj(path);
    let (models, materials) = obj_data.expect("error with file");
    let mut triangles = Vec::with_capacity(models.iter().map(|model|model.mesh.indices.len() / 3).sum());

    println!("# of models: {}", models.len());
    println!("# of materials: {}", materials.len());
    println!("# of triangles: {}", triangles.capacity());

    for model in models.iter() {
        println!("model.name: {}", model.name);
        let mesh = &model.mesh;
        let positions: Vec<_> = mesh.positions.chunks(3).map(|i| Point3::<f32>::new(i[0], i[1], i[2])).collect();
        let normals: Vec<_> = mesh.normals.chunks(3).map(|i| Vector3::<f32>::new(i[0], i[1], i[2])).collect();
        let texcoords: Vec<_> = mesh.texcoords.chunks(2).map(|i| Point2::<f32>::new(i[0], i[1])).collect();

        println!("# of positions: {}", positions.len());
        println!("# of normals: {}", normals.len());
        println!("# of texture coördinates: {}", texcoords.len());

        triangles.extend(mesh.indices.chunks(3).map(|indexes|
            Triangle{
                position0: positions[indexes[0] as usize],
                position1: positions[indexes[1] as usize],
                position2: positions[indexes[2] as usize],
                normal0: normals[indexes[0] as usize],
                normal1: normals[indexes[1] as usize],
                normal2: normals[indexes[2] as usize],
                //texture0: texcoords[indexes[0] as usize],
                //texture1: texcoords[indexes[1] as usize],
                //texture2: texcoords[indexes[2] as usize],
                material: {
                    if let Some(material_id) = mesh.material_id {
                        let m = &materials[material_id];
                        if m.dissolve < 1.0 {
                            Material::Dielectric{
                                refraction_index_n1: 1.0,
                                refraction_index_n2: 1.3,
                                color: Vector3::new(m.ambient[0], m.ambient[1], m.ambient[2]),
                            }
                        } else {
                            Material::Diffuse{
                                speculaty: m.shininess,
                                color: Vector3::new(m.diffuse[0], m.diffuse[1], m.diffuse[2]),
                            }
                        }
                    } else {
                        //Material::Dielectric{
                        //    refraction_index_n1: 1.0,
                        //    refraction_index_n2: 1.3,
                        //    color: Vector3::new(0.9,0.8,0.7),
                        //}
                        Material::Diffuse {
                            speculaty: 0.,
                            color: Vector3::new(0.9,0.9,0.9),
                        }
                    }
                },
        }));
    }

    println!("# of triangles loaded: {}", triangles.len());

    triangles
    //for (i, m) in materials.iter().enumerate() {
    //    println!("material[{}].name = \'{}\'", i, m.name);
    //    println!("    material.Ka = ({}, {}, {})", m.ambient[0], m.ambient[1], m.ambient[2]);
    //    println!("    material.Kd = ({}, {}, {})", m.diffuse[0], m.diffuse[1], m.diffuse[2]);
    //    println!("    material.Ks = ({}, {}, {})", m.specular[0], m.specular[1], m.specular[2]);
    //    println!("    material.Ns = {}", m.shininess);
    //    println!("    material.d = {}", m.dissolve);
    //    println!("    material.map_Ka = {}", m.ambient_texture);
    //    println!("    material.map_Kd = {}", m.diffuse_texture);
    //    println!("    material.map_Ks = {}", m.specular_texture);
    //    println!("    material.map_Ns = {}", m.normal_texture);
    //    println!("    material.map_d = {}", m.dissolve_texture);
    //    for (k, v) in &m.unknown_param {
    //        println!("    material.{} = {}", k, v);
    //    }
    //}
}
