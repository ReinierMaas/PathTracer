extern crate cgmath;

use rand;
use std::f32;
use std::sync::Mutex;

use cgmath::Point3;
use primitive::Primitive;
use primitive::aabb::AABB;

use ray::{Ray,Intersection};

thread_local!(static NODE_STACK: Mutex<Vec<usize>> = Mutex::new(Vec::new()));

#[derive(Debug)]
struct BVHNode {
    bounds: AABB,
    left_first: u32,
    count: u32,
}

#[derive(Debug)]
pub struct BVH<T: Primitive> {
    objects: Vec<T>,
    indices: Vec<usize>,
    lights: Vec<usize>,
    bvh_nodes: Vec<BVHNode>,
}

impl<T: Primitive> BVH<T> {
    pub fn new(objects: Vec<T>) -> BVH<T> {
        let len = objects.len();
        let mut indices = Vec::with_capacity(len);
        let mut lights = Vec::new();
        for object in &objects {
            let count = indices.len();
            indices.push(count);
            if let Some(_) = object.is_light() {
                lights.push(count);
            }
        }
        println!("# of Lights: {}", lights.len());
        let mut bvh_nodes = Vec::with_capacity(len);
        bvh_nodes.push(BVHNode {
            bounds: objects.iter().fold(AABB::new(), |sum, val| sum.combine(&val.bounds())),
            left_first: 0,
            count: len as u32 }
        );
        let mut bvh = BVH {
            objects: objects,
            indices: indices,
            lights: lights,
            bvh_nodes: bvh_nodes,
        };
        bvh.subdivide();
        bvh
    }
    fn subdivide(&mut self) {
        NODE_STACK.with(|node_stack|node_stack.lock().unwrap().push(0)); // root node
        while let Some(node_index) = NODE_STACK.with(|node_stack|node_stack.lock().unwrap().pop()) {
            if self.bvh_nodes[node_index].count > 2 && self.partition(node_index) {
                let left = self.bvh_nodes[node_index].left_first as usize;
                NODE_STACK.with(|node_stack|node_stack.lock().unwrap().push(left + 1));
                NODE_STACK.with(|node_stack|node_stack.lock().unwrap().push(left));
            }
        }
    }
    fn partition(&mut self, node_index: usize) -> bool {
        let bounds = self.bvh_nodes[node_index].bounds;
        let first = self.bvh_nodes[node_index].left_first as usize;
        let count = self.bvh_nodes[node_index].count as usize;
        if let Some((axis, pivot)) = self.surface_area_heuristic(bounds, first, count) {
            // Best split axis selected
            let mut left_bound = AABB::new();
            let mut right_bound = AABB::new();

            let mut pivot_index = first;
            for index in first..first + count {
                let bound = self.objects[self.indices[index]].bounds();
                let centre_on_axis = self.objects[self.indices[index]].centre()[axis];
                if centre_on_axis <= pivot {
                    left_bound = left_bound.combine(&bound);
                    self.indices.swap(pivot_index, index);
                    pivot_index += 1;
                }
                else {
                    right_bound = right_bound.combine(&bound);
                }
            }

            //Split current node
            let left_index = self.bvh_nodes.len();
            self.bvh_nodes[node_index].left_first = left_index as u32;
            self.bvh_nodes[node_index].count = 0;

            self.bvh_nodes.push(
                BVHNode { bounds : left_bound, left_first : first as u32, count : (pivot_index - first) as u32 }
            );
            let left_count = self.bvh_nodes[left_index].count as usize;
            self.bvh_nodes.push(
                BVHNode { bounds : right_bound, left_first : pivot_index as u32, count : (count - left_count) as u32 }
            );

            true
        } else {
            false
        }
    }
    fn surface_area_heuristic(&self, bounds: AABB, first: usize, count: usize) -> Option<(usize, f32)> {
        let sah_parent = bounds.area() * count as f32;
        // Best split plane based on surface area heuristic
        let mut centre_bound = AABB::new();
        for index in first..first + count {
            let centre = &self.objects[self.indices[index]].centre();
            centre_bound = centre_bound.extent(centre);
        }
        let axis_length = centre_bound.size();
        let axis_delta = axis_length / 8.;

        let mut bounds = [AABB::new(); 24]; // 8 bounds times 3 axis
        let mut primitives = [0; 24];
        for index in first..first + count {
            let object = &self.objects[self.indices[index]];
            let centre = object.centre();
            let bound = object.bounds();

            for axis in 0..3 {
                let bin_index = axis * 8 +
                    // (centre on axis - minimum centre) / axis delta
                    (((centre[axis] - centre_bound.min[axis]) / axis_delta[axis])
                    // clamp between 0 and 7
                    .max(0.).min(7.) as usize);
                bounds[bin_index] = bounds[bin_index].combine(&bound);
                primitives[bin_index] += 1;
            }
        }

        let mut min_axis = None;
        let mut min_bin = None;
        let mut min_sah = sah_parent;
        for axis in 0..3 {
            let axis_offset = axis * 8;

            // calculate all boundaries and number of primitives from the right side
            let mut bounds_right = [AABB::new(); 8];
            let mut primitives_right = [0; 8];
            bounds_right[7] = bounds[axis_offset + 7];
            primitives_right[7] = primitives[axis_offset + 7];
            for bin_offset in (0..7).rev() {
                bounds_right[bin_offset] = bounds_right[bin_offset + 1].combine(&bounds[axis_offset + bin_offset]);
                primitives_right[bin_offset] = primitives_right[bin_offset + 1] + primitives[axis_offset + bin_offset];
            }

            // incrementaly calculate boundaries and number of primitives from the left side
            let mut bounds_left = AABB::new();
            let mut primitives_left = 0;
            for bin_offset in 0..8 {
                bounds_left = bounds_left.combine(&bounds[axis_offset + bin_offset]);
                primitives_left += primitives[axis_offset + bin_offset];
                let sah: f32 = primitives_left as f32 * bounds_left.area() + primitives_right[bin_offset] as f32 * bounds_right[bin_offset].area();

                // save lowest estimated intersection cost
                if sah < min_sah {
                    min_axis = Some(axis);
                    min_bin = Some(bin_offset);
                    min_sah = sah;
                }
            }
        }
        if let (Some(min_axis), Some(min_bin)) = (min_axis, min_bin) {
            Some((min_axis, centre_bound.min[min_axis] + (1 + min_bin) as f32 * axis_delta[min_axis]))
        } else {
            None
        }
        //// median split
        //let mut axis = 0;
        //for a in 1..3 {
        //    if axis_length[axis] < axis_length[a] {
        //        axis = a;
        //    }
        //}
        //let pivot = centre_bound.min[axis] + axis_length[axis] / 2.;
        //(axis, pivot)
    }
    pub fn intersect_closest(&self, ray: &mut Ray) -> Option<Intersection> {
        let mut closest_intersection = None;
        NODE_STACK.with(|node_stack|node_stack.lock().unwrap().push(0)); // root node
        while let Some(node_index) = NODE_STACK.with(|node_stack|node_stack.lock().unwrap().pop()) {
            let node = &self.bvh_nodes[node_index];
            if let Some(_) = node.bounds.intersect(ray) { // prune stack pops that don't get intersected anymore
                if node.count != 0 {
                    // leaf node
                    for index in node.left_first as usize..(node.left_first + node.count) as usize {
                        let object = &self.objects[self.indices[index]];
                        if let Some(intersection) = object.intersect(ray) {
                            closest_intersection = Some(intersection);
                        }
                    }
                } else {
                    // internal node
                    let left = node.left_first as usize;
                    let tl = self.bvh_nodes[left].bounds.intersect(ray);
                    let tr = self.bvh_nodes[left + 1].bounds.intersect(ray);
                    match (tl,tr) {
                        (Some((tlmin, _)), Some((trmin, _))) => if tlmin <= trmin { // push happens in the reverse order LIFO
                                    NODE_STACK.with(|node_stack|node_stack.lock().unwrap().push(left + 1));
                                    NODE_STACK.with(|node_stack|node_stack.lock().unwrap().push(left));
                                } else {
                                    NODE_STACK.with(|node_stack|node_stack.lock().unwrap().push(left));
                                    NODE_STACK.with(|node_stack|node_stack.lock().unwrap().push(left + 1));
                            },
                        (Some(_), None) => NODE_STACK.with(|node_stack|node_stack.lock().unwrap().push(left)),
                        (None, Some(_)) => NODE_STACK.with(|node_stack|node_stack.lock().unwrap().push(left + 1)),
                        (None, None) => {},
                    }
                }
            }
        }
        closest_intersection
    }
    pub fn intersect_any(&self, ray: &mut Ray) -> Option<Intersection> {
        NODE_STACK.with(|node_stack|node_stack.lock().unwrap().push(0)); // root node
        while let Some(node_index) = NODE_STACK.with(|node_stack|node_stack.lock().unwrap().pop()) {
            let node = &self.bvh_nodes[node_index];
            if let Some(_) = node.bounds.intersect(ray) { // prune stack pops that don't get intersected anymore
                if node.count != 0 {
                    // leaf node
                    for index in node.left_first as usize..(node.left_first + node.count) as usize {
                        let object = &self.objects[self.indices[index]];
                        if let Some(intersection) = object.intersect(ray) {
                            return Some(intersection);
                        }
                    }
                } else {
                    // internal node
                    let left = node.left_first as usize;
                    let tl = self.bvh_nodes[left].bounds.intersect(ray);
                    let tr = self.bvh_nodes[left + 1].bounds.intersect(ray);
                    match (tl,tr) {
                        (Some((tlmin, _)), Some((trmin, _))) => if tlmin <= trmin { // push happens in the reverse order LIFO
                                    NODE_STACK.with(|node_stack|node_stack.lock().unwrap().push(left + 1));
                                    NODE_STACK.with(|node_stack|node_stack.lock().unwrap().push(left));
                                } else {
                                    NODE_STACK.with(|node_stack|node_stack.lock().unwrap().push(left));
                                    NODE_STACK.with(|node_stack|node_stack.lock().unwrap().push(left + 1));
                            },
                        (Some(_), None) => NODE_STACK.with(|node_stack|node_stack.lock().unwrap().push(left)),
                        (None, Some(_)) => NODE_STACK.with(|node_stack|node_stack.lock().unwrap().push(left + 1)),
                        (None, None) => {},
                    }
                }
            }
        }
        None
    }

    pub fn random_light(&self) -> Option<(usize, &T)> {
        if self.lights.len() == 0 {
            None
        } else {
            use rand::distributions::*;
            let mut rng = rand::thread_rng();
            let index_range: Range<usize> = Range::new(0, self.lights.len());
            let i = index_range.ind_sample(&mut rng);
            let obj_idx = self.lights[i];
            Some((self.lights.len(), &self.objects[obj_idx]))
        }
    }
}
