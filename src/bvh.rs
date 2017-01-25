extern crate cgmath;

use rand;
use std::f32;

use cgmath::Point3;
use primitive::Primitive;
use primitive::aabb::AABB;

use ray::{Ray,Intersection};

#[derive(Debug)]
struct BVHNode {
    bounds: AABB,
    left_first: usize,
    count: usize,
}

#[derive(Debug)]
pub struct BVH<T: Primitive> {
    objects: Vec<T>,
    indices: Vec<usize>,
    lights: Vec<usize>,
    bvh_nodes: Vec<BVHNode>,
    reserve_capacity: usize,
}

impl<T: Primitive> BVH<T> {
    pub fn new(objects: Vec<T>) -> BVH<T> {
        let len = objects.len();
        let mut indices = Vec::with_capacity(len);
        let mut lights = Vec::with_capacity(len);
        for object in &objects {
            let count = indices.len();
            indices.push(count);
            if let Some(a) = object.is_light() {
                lights.push(count);
            }
        }
        let mut bvh_nodes = Vec::with_capacity(len);
        bvh_nodes.push(BVHNode {
            bounds: objects.iter().fold(AABB::new(), |sum, val| sum.combine(&val.bounds())),
            left_first: 0,
            count: len }
        );
        let mut bvh = BVH {
            objects: objects,
            indices: indices,
            lights: lights,
            bvh_nodes: bvh_nodes,
            reserve_capacity: 0,
        };
        bvh.subdivide(0);
        bvh.reserve_capacity = bvh.reserve_capacity();
        bvh
    }
    fn reserve_capacity(&self) -> usize {
        let mut node_stack = Vec::new();
        node_stack.push(0); // root node
        let mut reserve_capacity: usize = node_stack.len();
        while let Some(node_index) = node_stack.pop() {
            let node = &self.bvh_nodes[node_index];
            if node.count == 0 {
                // internal node
                node_stack.push(node.left_first + 1);
                node_stack.push(node.left_first);
                let len = node_stack.len();
                if len > reserve_capacity {
                    reserve_capacity = len;
                }
            }
        }
        println!("{:?}", reserve_capacity);
        reserve_capacity
    }
    fn subdivide(&mut self, node_index: usize) {
        if self.bvh_nodes[node_index].count > 2 && self.partition(node_index) {
            let left = self.bvh_nodes[node_index].left_first;
            self.subdivide(left);
            self.subdivide(left + 1);
        }
    }
    fn partition(&mut self, node_index: usize) -> bool {
        let first = self.bvh_nodes[node_index].left_first;
        let count = self.bvh_nodes[node_index].count;
        let (axis, pivot) = self.surface_area_heuristic(first, count);
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
        self.bvh_nodes[node_index].left_first = left_index;
        self.bvh_nodes[node_index].count = 0;

        self.bvh_nodes.push(
            BVHNode { bounds : left_bound, left_first : first, count : pivot_index - first }
        );
        let left_count = self.bvh_nodes[left_index].count;
        self.bvh_nodes.push(
            BVHNode { bounds : right_bound, left_first : pivot_index, count : count - left_count }
        );

        true
    }
    fn surface_area_heuristic(&self, first: usize, count: usize) -> (usize, f32) {
        // TODO: best split plane based on surface area heuristic
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

        let mut min_axis = 0;
        let mut min_bin = 0;
        let mut min_sah = f32::MAX;
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
                let sah = primitives_left as f32 * bounds_left.area() + primitives_right[bin_offset] as f32 * bounds_right[bin_offset].area();

                // save lowest estimated intersection cost
                if sah < min_sah {
                    min_axis = axis;
                    min_bin = bin_offset;
                    min_sah = sah;
                }
            }
        }
        (min_axis, centre_bound.min[min_axis] + (1 + min_bin) as f32 * axis_delta[min_axis])
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
        let mut node_stack = Vec::with_capacity(self.reserve_capacity);
        node_stack.push(0); // root node
        while let Some(node_index) = node_stack.pop() {
            let node = &self.bvh_nodes[node_index];
            if let Some(_) = node.bounds.intersect(ray) { // prune stack pops that don't get intersected anymore
                if node.count != 0 {
                    // leaf node
                    for index in node.left_first..node.left_first + node.count {
                        let object = &self.objects[self.indices[index]];
                        if let Some(intersection) = object.intersect(ray) {
                            closest_intersection = Some(intersection);
                        }
                    }
                } else {
                    // internal node
                    let tl = self.bvh_nodes[node.left_first].bounds.intersect(ray);
                    let tr = self.bvh_nodes[node.left_first + 1].bounds.intersect(ray);
                    match (tl,tr) {
                        (Some((tlmin, _)), Some((trmin, _))) => if tlmin <= trmin { // push happens in the reverse order LIFO
                                    node_stack.push(node.left_first + 1);
                                    node_stack.push(node.left_first);
                                } else {
                                    node_stack.push(node.left_first);
                                    node_stack.push(node.left_first + 1);
                            },
                        (Some(_), None) => node_stack.push(node.left_first),
                        (None, Some(_)) => node_stack.push(node.left_first + 1),
                        (None, None) => {},
                    }
                }
            }
        }
        closest_intersection
    }
    pub fn intersect_any(&self, ray: &mut Ray) -> Option<Intersection> {
        let mut node_stack = Vec::with_capacity(self.reserve_capacity);
        node_stack.push(0); // root node
        while let Some(node_index) = node_stack.pop() {
            let node = &self.bvh_nodes[node_index];
            if let Some(_) = node.bounds.intersect(ray) { // prune stack pops that don't get intersected anymore
                if node.count != 0 {
                    // leaf node
                    for index in node.left_first..node.left_first + node.count {
                        let object = &self.objects[self.indices[index]];
                        if let Some(intersection) = object.intersect(ray) {
                            return Some(intersection);
                        }
                    }
                } else {
                    // internal node
                    let tl = self.bvh_nodes[node.left_first].bounds.intersect(ray);
                    let tr = self.bvh_nodes[node.left_first + 1].bounds.intersect(ray);
                    match (tl,tr) {
                        (Some((tlmin, _)), Some((trmin, _))) => if tlmin <= trmin { // push happens in the reverse order LIFO
                                    node_stack.push(node.left_first + 1);
                                    node_stack.push(node.left_first);
                                } else {
                                    node_stack.push(node.left_first);
                                    node_stack.push(node.left_first + 1);
                            },
                        (Some(_), None) => node_stack.push(node.left_first),
                        (None, Some(_)) => node_stack.push(node.left_first + 1),
                        (None, None) => {},
                    }
                }
            }
        }
        None
    }

    pub fn random_light(&self) -> &T {
        use std::f32;
        use rand::distributions::*;
        let mut rng = rand::thread_rng();
        let index_range: Range<usize> = Range::new(0, self.lights.len());
        let i = index_range.ind_sample(&mut rng);
        let obj_idx = self.lights[i];
        &self.objects[obj_idx]
    }
}
