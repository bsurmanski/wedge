use super::index::*;

/*
 * Vertex
 */
#[derive(Copy, Clone)]
struct VertexInfo<Ix, V> {
    base_edge_index: Ix, // optional.
    data: V,
}

impl<Ix : IndexType, V> VertexInfo<Ix, V> {
    pub fn new(data : V) -> Self {
       VertexInfo { base_edge_index: Ix::max(), data: data } 
    }
}

#[derive(Copy, Clone)]
pub struct VertexRef<'a, V, E, F> {
    mesh: &'a Mesh<V, E, F>,
    vertex_index: Index,
}

impl<'a, V, E, F> VertexRef<'a, V, E, F> {
    // Public methods 
    pub fn new(mesh: &'a Mesh<V, E, F>, index: Index) -> Self {
        VertexRef{ mesh: mesh, vertex_index: index }
    }

    pub fn data(&self) -> &'a V {
        return &self.vertex_info().data;
    }

    pub fn index(&self) -> Index {
        return self.vertex_index;
    }

    pub fn edge_iter(&self) -> VertexEdgeIterator<V, E, F> {
        let vertex_info = self.mesh.vertex_info(self.vertex_index);
        return VertexEdgeIterator { 
            mesh: self.mesh,
            base_vertex_index: self.vertex_index,
            start_edge_index: vertex_info.base_edge_index.to_option(),
            current_edge_index: vertex_info.base_edge_index.to_option(),
        };
    }

    pub fn face_iter(&self) -> VertexFaceIterator<V, E, F> {
        return VertexFaceIterator { 
            edge_iter: self.edge_iter()
        };
    }

    // Private methods
    fn vertex_info(&self) -> &'a VertexInfo<Index, V> {
        return &self.mesh.vertex_info(self.vertex_index);
    }
}

#[derive(Copy, Clone)]
pub struct VertexEdgeIterator<'a, V, E, F> {
    mesh: &'a Mesh<V, E, F>,
    base_vertex_index: Index,
    start_edge_index: Option<Index>,
    current_edge_index: Option<Index>,
}

impl<'a, V, E, F> VertexEdgeIterator<'a, V, E, F> {
    pub fn vertex(&self) -> VertexRef<'a, V, E, F> {
        self.mesh.vertex(self.base_vertex_index)
    }

    pub fn start_edge(&self) -> Option<EdgeRef<'a, V, E, F>> {
        match self.start_edge_index {
            Some(index) => Some(EdgeRef{mesh: self.mesh, edge_index: index }),
            None => None
        }
    }
    
    fn current_edge(&self) -> Option<&EdgeInfo<E>> {
        match self.current_edge_index {
            Some(index) => Some(self.mesh.edge_info(index)),
            None => None
        }
    }
}

impl<'a, V, E, F> Iterator for VertexEdgeIterator<'a, V, E, F> {
    type Item = EdgeRef<'a, V, E, F>;

    // Iterates over the edges of a vertex.
    fn next(&mut self) -> Option<EdgeRef<'a, V, E, F>> {
        match self.current_edge() {
            Some(edge) => {
                let next_edge_index: Index;
                    if edge.half_edge[0].vertex_index == self.base_vertex_index {
                        next_edge_index = edge.half_edge[0].next_edge_index;
                } else {
                    assert!(edge.half_edge[1].vertex_index == self.base_vertex_index,
                            "edge iterator reached an edge unconnected to the base vertex!");
                        next_edge_index = edge.half_edge[1].next_edge_index;

                }

                // it doesn't make sense to have a next edge without a start edge,
                // so using unwrap is fine here.
                if self.start_edge_index.unwrap() == next_edge_index {
                    self.current_edge_index = None;
                    return None;
                } else {
                    self.current_edge_index = Some(next_edge_index);
                    return Some(EdgeRef {
                        mesh: self.mesh,
                        edge_index: next_edge_index,
                    });
                }
            },
            None => return None
        }
    }
}

#[derive(Copy, Clone)]
pub struct VertexFaceIterator<'a, V, E, F> {
    edge_iter: VertexEdgeIterator<'a, V, E, F>,
}

impl<'a, V, E, F> Iterator for VertexFaceIterator<'a, V, E, F> {
    type Item = FaceRef<'a, V, E, F>;

    fn next(&mut self) -> Option<FaceRef<'a, V, E, F>> {
        let maybe_edge = self.edge_iter.next();
        match maybe_edge {
            Some(edge) => {
                let face_index: Index;
                if edge.edge_info().half_edge[0].vertex_index == self.edge_iter.base_vertex_index {
                    face_index = edge.edge_info().half_edge[0].next_face_index;
                } else {
                    assert!(edge.edge_info().half_edge[1].vertex_index == self.edge_iter.base_vertex_index,
                            "face iterator reached an face unconnected to the base vertex!");
                    face_index = edge.edge_info().half_edge[1].next_face_index;
                }
                return Some(FaceRef {
                    mesh: self.edge_iter.mesh,
                    face_index: face_index,
                });
            },
            None => None
        }
    }
}

/* 
 * Edges 
 */
#[derive(Copy, Clone)]
struct HalfEdgeInfo {
    vertex_index: Index,    // required.
    next_face_index: Index, // optional. cw relative to base vertex
    next_edge_index: Index, // optional. cw around base vertex
    prev_edge_index: Index, // optional. ccw around base vertex
}

#[derive(Copy, Clone)]
struct EdgeInfo<E> {
    half_edge: [HalfEdgeInfo; 2],
    data: E,
}

#[derive(Copy, Clone)]
pub struct EdgeRef<'a, V, E, F> {
    mesh: &'a Mesh<V, E, F>,
    edge_index: Index 
}

impl<'a, V, E, F> EdgeRef<'a, V, E, F> {
    pub fn new(mesh: &'a Mesh<V, E, F>, index: Index) -> EdgeRef<V, E, F> {
        EdgeRef { mesh: mesh, edge_index: index }
    }

    fn edge_info(&self) -> &'a EdgeInfo<E> {
        return &self.mesh.edge_info(self.edge_index);
    }

    pub fn data(&self) -> &'a E {
        return &self.edge_info().data
    }

    // TODO: in non-manifold meshes, an edge might have <2 faces.
    pub fn faces(&self) -> Vec<FaceRef<'a, V, E, F>> {
        let edge_info = self.edge_info();
        vec![
            FaceRef{mesh: self.mesh, face_index: edge_info.half_edge[0].next_face_index},
            FaceRef{mesh: self.mesh, face_index: edge_info.half_edge[1].next_face_index},
        ]
    }

    pub fn vertices(&self) -> Vec<VertexRef<'a, V, E, F>> {
        let edge_info = self.edge_info();
        vec![
            VertexRef{mesh: self.mesh, vertex_index: edge_info.half_edge[0].vertex_index},
            VertexRef{mesh: self.mesh, vertex_index: edge_info.half_edge[1].vertex_index},
        ]
    }
}

/*
 * Faces 
 */
#[derive(Copy, Clone)]
struct FaceInfo<F> {
    base_edge_index: Index, // required.
    data: F,
}

#[derive(Copy, Clone)]
pub struct FaceRef<'a, V, E, F> {
    mesh: &'a Mesh<V, E, F>,
    face_index: Index,
}

impl<'a, V, E, F> FaceRef<'a, V, E, F> {
    fn face_info(&self) -> &'a FaceInfo<F> {
        return &self.mesh.face_info(self.face_index);
    }

    pub fn data(&self) -> &F {
        return &self.face_info().data
    }

    pub fn edge_iter(&self) -> FaceEdgeIterator<'a, V, E, F> {
        let edge_index = self.face_info().base_edge_index;
        assert!(self.mesh.is_valid_edge_index(edge_index));
        return FaceEdgeIterator {
            mesh: self.mesh,
            face_index: self.face_index,
            start_edge_index: edge_index,
            current_edge_index: edge_index,
        }
    }
}

#[allow(dead_code)] // TODO: remove when implemented
#[derive(Copy, Clone)]
pub struct FaceEdgeIterator<'a, V, E, F> {
    mesh: &'a Mesh<V, E, F>,
    face_index: Index,
    start_edge_index: Index,
    current_edge_index: Index,
}

impl<'a, V, E, F> Iterator for FaceEdgeIterator<'a, V, E, F> {
    type Item = EdgeRef<'a, V, E, F>;

    fn next(&mut self) -> Option<EdgeRef<'a, V, E, F>> {
        unimplemented!(); // TODO
    }
}

#[allow(dead_code)] // TODO: remove when implemented
#[derive(Copy, Clone)]
pub struct FaceVertexIterator<'a, V, E, F> {
    edge_iter: FaceEdgeIterator<'a, V, E, F>,
}

impl<'a, V, E, F> Iterator for FaceVertexIterator<'a, V, E, F> {
    type Item = VertexRef<'a, V, E, F>;

    fn next(&mut self) -> Option<VertexRef<'a, V, E, F>> {
        unimplemented!(); // TODO
    }
}

/*
 * Mesh
 */
#[derive(Clone)]
pub struct Mesh<V, E, F> {
    verts: Vec<VertexInfo<Index, V>>,
    edges: Vec<EdgeInfo<E>>,
    faces: Vec<FaceInfo<F>>,
}

impl<V, E, F> Mesh<V, E, F> {
    pub fn new() -> Mesh<V, E, F> {
        Mesh {
            verts: Vec::new(),
            edges: Vec::new(),
            faces: Vec::new()
        }
    }

    pub fn is_valid_vert_index(&self, index: Index) -> bool {
        return index < Index::max_value() && (index as usize) < self.verts.len();
    }

    pub fn is_valid_edge_index(&self, index: Index) -> bool {
        return index < Index::max_value() && (index as usize) < self.edges.len();
    }

    pub fn is_valid_face_index(&self, index: Index) -> bool {
        return index < Index::max_value() && (index as usize) < self.faces.len();
    }

    pub fn vertex(&self, index: Index) -> VertexRef<V, E, F> {
        return VertexRef{mesh: self, vertex_index: index}
    }

    fn vertex_info(&self, index: Index) -> &VertexInfo<Index, V> {
        return &self.verts[index as usize];
    }

    fn edge_info(&self, index: Index) -> &EdgeInfo<E> {
        return &self.edges[index as usize];
    }

    fn face_info(&self, index: Index) -> &FaceInfo<F> {
        return &self.faces[index as usize];
    }
    
    pub fn edge(&self, index: Index) -> EdgeRef<V, E, F> {
        return EdgeRef{mesh: self, edge_index: index}
    }

    pub fn face(&self, index: Index) -> FaceRef<V, E, F> {
        return FaceRef{mesh: self, face_index: index}
    }

    pub fn vertex_iter(&self) -> MeshVertexIterator<V, E, F> {
        MeshVertexIterator { mesh: self, vertex_index: 0 }
    }

    pub fn edge_iter(&self) -> MeshEdgeIterator<V, E, F> {
        MeshEdgeIterator { mesh: self, edge_index: 0 }
    }

    pub fn face_iter(&self) -> MeshFaceIterator<V, E, F> {
        MeshFaceIterator { mesh: self, face_index: 0 }
    }

    pub fn add_vertex(&mut self, v: V) -> VertexRef<V, E, F> {
        let index = Index::new(self.verts.len());
        self.verts.push(VertexInfo::new(v));
        return VertexRef::new(self, index);
    }

    pub fn add_edge(&mut self, v1: VertexRef<V, E, F>, v2: VertexRef<V, E, F>) -> EdgeRef<V, E, F> {
        let index = Index::new(self.edges.len());
       
        unimplemented!(); 
        // PUSH EDGE 
        return EdgeRef::new(self, index);
    }
}

#[derive(Copy, Clone)]
pub struct MeshVertexIterator<'a, V, E, F> {
    mesh: &'a Mesh<V, E, F>,
    vertex_index: Index,
}

impl<'a, V, E, F> Iterator for MeshVertexIterator<'a, V, E, F> {
    type Item = VertexRef<'a, V, E, F>;

    fn next(&mut self) -> Option<VertexRef<'a, V, E, F>> {
        if self.mesh.is_valid_vert_index(self.vertex_index) {
            let ret = Some(VertexRef { mesh: self.mesh, vertex_index: self.vertex_index });
            self.vertex_index += 1;
            return ret;
        }
        return None;
    }
}

#[derive(Copy, Clone)]
pub struct MeshEdgeIterator<'a, V, E, F> {
    mesh: &'a Mesh<V, E, F>,
    edge_index: Index,
}

impl<'a, V, E, F> Iterator for MeshEdgeIterator<'a, V, E, F> {
    type Item = EdgeRef<'a, V, E, F>;

    fn next(&mut self) -> Option<EdgeRef<'a, V, E, F>> {
        if self.mesh.is_valid_edge_index(self.edge_index) {
            let ret = Some(EdgeRef { mesh: self.mesh, edge_index: self.edge_index });
            self.edge_index += 1;
            return ret;
        }
        return None;
    }
}

#[derive(Copy, Clone)]
pub struct MeshFaceIterator<'a, V, E, F> {
    mesh: &'a Mesh<V, E, F>,
    face_index: Index,
}

impl<'a, V, E, F> Iterator for MeshFaceIterator<'a, V, E, F> {
    type Item = FaceRef<'a, V, E, F>;

    fn next(&mut self) -> Option<FaceRef<'a, V, E, F>> {
        if self.mesh.is_valid_face_index(self.face_index) {
            let ret = Some(FaceRef { mesh: self.mesh, face_index: self.face_index });
            self.face_index += 1;
            return ret;
        }
        return None;
    }
}
