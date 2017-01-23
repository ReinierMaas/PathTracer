extern crate cgmath;

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
}

impl<T: Primitive> BVH<T> {
    pub fn new(objects: Vec<T>) -> BVH<T> {
        let len = objects.len();
        println!("{:?}", len);
        let mut indices = Vec::with_capacity(len);
        let mut lights = Vec::with_capacity(len);
        for object in &objects {
            let count = indices.len();
            indices.push(count);
            if object.is_light() {
                lights.push(count);
            }
        }
        let mut bvh_nodes = Vec::with_capacity(len);
        bvh_nodes.push(BVHNode {
            bounds: objects.iter().fold(AABB::new(), |sum, val| sum.combine(&val.bounds())),
            left_first: 0,
            count: len }
        );
        println!("{:?}", bvh_nodes[0].bounds);
        let mut bvh = BVH {
            objects: objects,
            indices: indices,
            lights: lights,
            bvh_nodes: bvh_nodes
        };
        bvh.subdivide(0);
        bvh
    }
    fn subdivide(&mut self, node_index: usize) {
        if self.bvh_nodes[node_index].count > 2 && self.partition(node_index) {
            let left = self.bvh_nodes[node_index].left_first;
            self.subdivide(left);
            self.subdivide(left + 1);
        }
    }
    fn partition(&mut self, node_index: usize) -> bool {
        let mut centre_bound = AABB::new();
        let first = self.bvh_nodes[node_index].left_first;
        let count = self.bvh_nodes[node_index].count;
        for index in first..count {
            let centre = &self.objects[self.indices[index]].centre();
            centre_bound = centre_bound.extent(centre);
        }
        let axis_length = centre_bound.size();
        let mut axis = 0;
        for a in 1..3 {
            if axis_length[axis] < axis_length[a] {
                axis = a;
            }
        }
        let axis = axis; // make immutable
        let pivot = centre_bound.min[axis] + axis_length[axis] / 2.;
        // TODO: best split plane based on surface area heuristic

        // Best split axis selected
        let mut left_bound = AABB::new();
        let mut right_bound = AABB::new();

        let mut pivot_index = first;
        for index in first..count {
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

        //assert!(self.bvh_nodes[node_index].bounds >= self.bvh_nodes[self.bvh_nodes[node_index].left_first].bounds);
        //assert!(self.bvh_nodes[node_index].bounds >= self.bvh_nodes[self.bvh_nodes[node_index].left_first + 1].bounds);

        println!("{:?}", count);
        println!("{:?}", (node_index, &self.bvh_nodes[node_index]));
        println!("{:?}", (self.bvh_nodes[node_index].left_first, &self.bvh_nodes[self.bvh_nodes[node_index].left_first]));
        println!("{:?}", (self.bvh_nodes[node_index].left_first + 1, &self.bvh_nodes[self.bvh_nodes[node_index].left_first + 1]));

        true
    }
    pub fn intersect(&self, ray: &mut Ray) -> Option<Intersection> {
        let mut closest_intersection = None;
        let mut node_stack = Vec::new();
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
}
