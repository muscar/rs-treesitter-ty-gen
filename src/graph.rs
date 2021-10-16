mod graph;

#[derive(Clone, Copy, Debug)]
pub struct VertexId(usize);

pub struct Vertex<'a, T> {
    id: VertexId,
    value: &'a T,
}

pub struct Graph<T> {
    vertices: Vec<T>,
    adj_list: Vec<Vec<usize>>,
}

impl<T> Graph<T> {
    pub fn new() -> Self {
        Self {
            vertices: Vec::new(),
            adj_list: Vec::new(),
        }
    }

    pub fn add_vertex(&mut self, t: T) -> VertexId {
        let id = VertexId(self.vertices.len());
        self.vertices.push(t);
        self.adj_list.push(Vec::new());
        id
    }

    pub fn add_edge(&mut self, u: VertexId, v: VertexId) {
        self.adj_list[u.0].push(v.0);
    }

    pub fn get_vertex(&self, id: VertexId) -> Vertex<T> {
        Vertex { id, value: &self.vertices[id.0] }
    }

    pub fn get_out_edges(&self, id: VertexId) -> Vec<VertexId> {
        self.adj_list[id.0].iter().map(|u| VertexId(*u)).collect()
    }

    pub fn vertex_count(&self) -> usize {
        self.vertices.len()
    }

    pub fn vertices(&self) -> VertexIterator<T> {
        VertexIterator { graph: self, curr: 0 }
    }
}

pub struct VertexIterator<'a, T> {
    graph: &'a Graph<T>,
    curr: usize,
}

impl<'a, T> Iterator for VertexIterator<'a, T> {
    type Item = Vertex<'a, T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.curr >= self.graph.vertices.len() {
            return None
        }
        let v = self.graph.get_vertex(VertexId(self.curr));
        self.curr += 1;
        Some(v)
    }
}

pub fn topo_sort<'a, T>(g: &'a Graph<T>) -> Vec<&'a T> {
    let mut in_degree = vec![0; g.vertex_count()];
    let mut next = Vec::new();
    let mut order = Vec::new();
    for v in g.vertices() {
        for u in g.get_out_edges(v.id) {
            in_degree[u.0] += 1;
        }
    }
    for (i, d) in in_degree.iter().enumerate() {
        if *d == 0 {
            next.push(VertexId(i));
        }
    }
    while let Some(u) = next.pop() {
        order.push(g.get_vertex(u).value);
        for v in g.get_out_edges(u) {
            in_degree[v.0] -= 1;
            if in_degree[v.0] == 0 {
                next.push(v);
            }
        }
    }
    order
}

// fn main() {
//     let mut g = Graph::new();
//     let a = g.add_vertex('A');
//     let b = g.add_vertex('B');
//     let c = g.add_vertex('C');
//     let d= g.add_vertex('D');
//     let e= g.add_vertex('E');
//     let f= g.add_vertex('F');
//     g.add_edge(a, f);
//     g.add_edge(b, a);
//     g.add_edge(d, b);
//     g.add_edge(d, c);
//     g.add_edge(e, c);
//     g.add_edge(e, f);
//     for v in g.vertices() {
//         println!("{:?} -> {:?}", v.value, g.get_out_edges(v.id).iter().map(|u| g.get_vertex(*u).value).collect::<Vec<&char>>());
//     }
//     let o = topo_sort(&g);
//     for x in o {
//         println!("{}", x);
//     }
// }
