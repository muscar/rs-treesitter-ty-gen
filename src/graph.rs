use std::{
    collections::BinaryHeap,
    fmt::{self, Display},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct VertexId(usize);

pub struct Vertex<'a, T, W: Ord> {
    pub id: VertexId,
    pub value: &'a T,
    pub weight: &'a W,
}

impl<'a, T, W: Ord> Ord for Vertex<'a, T, W> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.weight.cmp(other.weight)
    }
}

impl<'a, T, W: Ord> PartialOrd for Vertex<'a, T, W> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.weight.partial_cmp(other.weight)
    }
}

impl<'a, T, W: Ord> Eq for Vertex<'a, T, W> {}

impl<'a, T, W: Ord> PartialEq for Vertex<'a, T, W> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

pub struct Graph<T, W: Ord> {
    vertices: Vec<(T, W)>,
    adj_list: Vec<Vec<usize>>,
}

impl<T, W: Ord> Graph<T, W> {
    pub fn new() -> Self {
        Self {
            vertices: Vec::new(),
            adj_list: Vec::new(),
        }
    }

    pub fn add_vertex(&mut self, t: T, weight: W) -> VertexId {
        let id = VertexId(self.vertices.len());
        self.vertices.push((t, weight));
        self.adj_list.push(Vec::new());
        id
    }

    pub fn add_edge(&mut self, u: VertexId, v: VertexId) {
        self.adj_list[u.0].push(v.0);
    }

    pub fn get_vertex(&self, id: VertexId) -> Vertex<T, W> {
        let v = &self.vertices[id.0];
        Vertex {
            id,
            value: &v.0,
            weight: &v.1,
        }
    }

    pub fn get_out_edges(&self, id: VertexId) -> Vec<VertexId> {
        self.adj_list[id.0].iter().map(|u| VertexId(*u)).collect()
    }

    pub fn vertex_count(&self) -> usize {
        self.vertices.len()
    }

    pub fn vertices(&self) -> VertexIterator<T, W> {
        VertexIterator {
            graph: self,
            curr_idx: 0,
        }
    }
}

pub struct VertexIterator<'a, T, W: Ord> {
    graph: &'a Graph<T, W>,
    curr_idx: usize,
}

impl<'a, T, W: Ord> Iterator for VertexIterator<'a, T, W> {
    type Item = Vertex<'a, T, W>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.curr_idx >= self.graph.vertices.len() {
            return None;
        }
        let v = self.graph.get_vertex(VertexId(self.curr_idx));
        self.curr_idx += 1;
        Some(v)
    }
}

impl<T: Display, W: Ord> Display for Graph<T, W> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for v in self.vertices() {
            writeln!(f, "{} -> [", v.value)?;
            let es = self.get_out_edges(v.id);
            let mut it = es.iter();
            if let Some(x) = it.next() {
                self.get_vertex(*x).value.fmt(f)?;
                for t in it {
                    write!(f, ", ")?;
                    fmt::Display::fmt(&self.get_vertex(*t).value, f)?;
                }
            }
            write!(f, "]")?;
        }
        Ok(())
    }
}

pub fn topo_sort<T, W: Ord>(g: &Graph<T, W>) -> Vec<&T> {
    let mut in_degree = vec![0; g.vertex_count()];
    let mut next = BinaryHeap::new();
    let mut order = Vec::new();
    for v in g.vertices() {
        for u in g.get_out_edges(v.id) {
            in_degree[u.0] += 1;
        }
    }
    for (i, d) in in_degree.iter().enumerate() {
        if *d == 0 {
            next.push(g.get_vertex(VertexId(i)));
        }
    }
    while let Some(u) = next.pop() {
        order.push(u.value);
        for v in g.get_out_edges(u.id) {
            in_degree[v.0] -= 1;
            if in_degree[v.0] == 0 {
                next.push(g.get_vertex(v));
            }
        }
    }
    order
}
