extern crate cgmath;
use self::cgmath::Point3;

use primitive::Primitive;
use primitive::aabb::AABB;
use primitive::triangle::Triangle;
use primitive::sphere::Sphere;


#[derive(Debug)]
struct BVHNode {
    bounds: AABB,
    left_first: usize,
    count: usize,
}

#[derive(Debug)]
pub struct BVH {
    triangles: Vec<Triangle>,
    spheres: Vec<Sphere>,
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

impl BVH {
    pub fn new(triangles: Vec<Triangle>, spheres: Vec<Sphere>) -> BVH {
        let total = triangles.len() + spheres.len();
        let mut indices = Vec::with_capacity(total);
        let mut bounds = Vec::with_capacity(total);
        let mut centres = Vec::with_capacity(total);
        let mut lights = Vec::with_capacity(total);
        let mut count = 0;
        for triangle in &triangles {
            indices.push(count);
            bounds.push(triangle.bounds());
            centres.push(triangle.centre());
            if triangle.is_light() {
                lights.push(count);
            }
            count += 1;
        }
        for sphere in &spheres {
            indices.push(count);
            bounds.push(sphere.bounds());
            centres.push(sphere.centre());
            if sphere.is_light() {
                lights.push(count);
            }
            count += 1;
        }
        let mut bvh_nodes = Vec::with_capacity(total);
        bvh_nodes.push(BVHNode {
            bounds: bounds.iter().fold(AABB::new(), |sum, val| sum.combine(val)),
            left_first: 0,
            count: total }
        );
        let mut bvh = BVH {
            triangles: triangles,
            spheres: spheres,
            indices: indices,
            bounds: bounds,
            centres: centres,
            lights: lights,
            bvh_nodes: bvh_nodes
        };
        bvh.subdivide(0);
        bvh
    }
    fn subdivide(&mut self, node_index: usize) {
        if self.bvh_nodes[node_index].count < 3 {
            return;
        }
        if self.partition(node_index) {
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
}
