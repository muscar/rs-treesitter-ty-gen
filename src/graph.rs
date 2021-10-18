use std::{
    collections::VecDeque,
    fmt::{self, Display},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct VertexId(usize);

pub struct Vertex<'a, T> {
    pub id: VertexId,
    pub value: &'a T,
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
        Vertex {
            id,
            value: &self.vertices[id.0],
        }
    }

    pub fn get_out_edges(&self, id: VertexId) -> Vec<VertexId> {
        self.adj_list[id.0].iter().map(|u| VertexId(*u)).collect()
    }

    pub fn vertex_count(&self) -> usize {
        self.vertices.len()
    }

    pub fn vertices(&self) -> VertexIterator<T> {
        VertexIterator {
            graph: self,
            curr_idx: 0,
        }
    }
}

pub struct VertexIterator<'a, T> {
    graph: &'a Graph<T>,
    curr_idx: usize,
}

impl<'a, T> Iterator for VertexIterator<'a, T> {
    type Item = Vertex<'a, T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.curr_idx >= self.graph.vertices.len() {
            return None;
        }
        let v = self.graph.get_vertex(VertexId(self.curr_idx));
        self.curr_idx += 1;
        Some(v)
    }
}

impl<T: Display> Display for Graph<T> {
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

pub fn topo_sort<T>(g: &Graph<T>) -> Vec<&T> {
    let mut in_degree = vec![0; g.vertex_count()];
    let mut next = VecDeque::new();
    let mut order = Vec::new();
    for v in g.vertices() {
        for u in g.get_out_edges(v.id) {
            in_degree[u.0] += 1;
        }
    }
    for (i, d) in in_degree.iter().enumerate() {
        if *d == 0 {
            next.push_back(g.get_vertex(VertexId(i)));
        }
    }
    while let Some(u) = next.pop_front() {
        order.push(u.value);
        for v in g.get_out_edges(u.id) {
            in_degree[v.0] -= 1;
            if in_degree[v.0] == 0 {
                next.push_back(g.get_vertex(v));
            }
        }
    }
    order
}
