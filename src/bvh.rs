extern crate cgmath;
use self::cgmath::Point3;

use primitive::Primitive;
use primitive::aabb::AABB;
use primitive::triangle::Triangle;
use primitive::sphere::Sphere;

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
    bounds: Vec<AABB>,
    centres: Vec<Point3<f32>>,
    lights: Vec<usize>,
    bvh_nodes: Vec<BVHNode>,
}

enum Index {
    Triangle(usize),
    Sphere(usize),
}

impl<T: Primitive> BVH<T> {
    pub fn new(objects: Vec<T>) -> BVH<T> {
        let len = objects.len();
        let mut indices = Vec::with_capacity(len);
        let mut bounds = Vec::with_capacity(len);
        let mut centres = Vec::with_capacity(len);
        let mut lights = Vec::with_capacity(len);
        let mut count = 0;
        for object in &objects {
            indices.push(count);
            bounds.push(object.bounds());
            centres.push(object.centre());
            if object.is_light() {
                lights.push(count);
            }
            count += 1;
        }
        let mut bvh_nodes = Vec::with_capacity(len);
        bvh_nodes.push(BVHNode {
            bounds: bounds.iter().fold(AABB::new(), |sum, val| sum.combine(val)),
            left_first: 0,
            count: len }
        );
        let mut bvh = BVH {
            objects: objects,
            indices: indices,
            bounds: bounds,
            centres: centres,
            lights: lights,
            bvh_nodes: bvh_nodes
        };
        //bvh.subdivide(0);
        bvh
    }
    fn subdivide(&mut self, node_index: usize) {
        if self.bvh_nodes[node_index].count > 2 && self.partition(node_index) {
            let left = self.bvh_nodes[node_index].left_first;
            //self.subdivide(left);
            //self.subdivide(left + 1);
        }
    }
    fn partition(&mut self, node_index: usize) -> bool {
        let mut centre_bound = AABB::new();
        let first = self.bvh_nodes[node_index].left_first;
        let count = self.bvh_nodes[node_index].count;
        for index in first..count {
            let centre = &self.centres[self.indices[index]];
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
            let bound = &self.bounds[self.indices[index]];
            let centre_on_axis = self.centres[self.indices[index]][axis];
            if centre_on_axis <= pivot {
                left_bound = left_bound.combine(bound);
                self.indices.swap(pivot_index, index);
                pivot_index += 1;
            }
            else {
                right_bound = right_bound.combine(bound);
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
    pub fn intersect(&self, ray: &mut Ray) -> Option<Intersection> {
        let mut closest_intersection = None;
        let mut node_stack = Vec::new();
        node_stack.push(0); // root node
        while let Some(node_index) = node_stack.pop() {
            if self.bvh_nodes[node_index].count != 0 {
                // leaf node
                for index in self.bvh_nodes[node_index].left_first..self.bvh_nodes[node_index].count {
                    let object = &self.objects[self.indices[index]];
                    if let Some(intersection) = object.intersect(ray) {
                        closest_intersection = Some(intersection);
                    }
                }
            } else {
                // internal node
                let tl = self.bvh_nodes[self.bvh_nodes[node_index].left_first].bounds.intersect(ray);
                let tr = self.bvh_nodes[self.bvh_nodes[node_index].left_first + 1].bounds.intersect(ray);
                match (tl,tr) {
                    (Some((tlmin, _)), Some((trmin, _))) => if tlmin <= trmin {
                            node_stack.push(self.bvh_nodes[node_index].left_first);
                            node_stack.push(self.bvh_nodes[node_index].left_first + 1);
                        } else {
                            node_stack.push(self.bvh_nodes[node_index].left_first + 1);
                            node_stack.push(self.bvh_nodes[node_index].left_first);
                        },
                    (Some(_), None) => node_stack.push(self.bvh_nodes[node_index].left_first),
                    (None, Some(_)) => node_stack.push(self.bvh_nodes[node_index].left_first + 1),
                    (None, None) => {},
                }
            }
        }
        closest_intersection
    }
}
