type Index = u32;

pub trait IndexType : Copy + Default + Ord {
    fn new(x: usize) -> Self;
    fn index(&self) -> usize;
    fn max(&self) -> Self;
    fn is_valid(&self) -> bool {
        return *self != self.max();
    }
    fn to_option(&self) -> Option<Self> {
        if self.is_valid() {
            return None;
        }
        return Some(*self);
    }
}

impl IndexType for u32 {
    fn new(x: usize) -> Self {
        return x as Self;
    }

    fn index(&self) -> usize {
        return *self as usize;
    }

    fn max(&self) -> Self {
        return Self::max_value();
    }
}

fn index_to_option(i: Index) -> Option<Index> {
    if i == Index::max_value() {
        return None;
    }
    return Some(i);
}

/*
 * Vertex
 */
struct VertexInfo<VD> {
    base_edge_index: Index, // optional.
    data: VD,
}

pub struct VertexRef<'a, VD, ED, FD> {
    mesh: &'a Mesh<VD, ED, FD>,
    vertex_index: Index,
}

impl<'a, VD, ED, FD> VertexRef<'a, VD, ED, FD> {
    // Public methods 
    pub fn data(&self) -> &'a VD {
        return &self.vertex_info().data;
    }

    pub fn index(&self) -> Index {
        return self.vertex_index;
    }

    pub fn edge_iter(&self) -> VertexEdgeIterator<VD, ED, FD> {
        let vertex_info = &self.mesh.verts[self.vertex_index as usize];
        return VertexEdgeIterator { 
            mesh: self.mesh,
            base_vertex_index: self.vertex_index,
            start_edge_index: index_to_option(vertex_info.base_edge_index),
            current_edge_index: index_to_option(vertex_info.base_edge_index),
        };
    }

    pub fn face_iter(&self) -> VertexFaceIterator<VD, ED, FD> {
        return VertexFaceIterator { 
            edge_iter: self.edge_iter()
        };
    }

    // Private methods
    fn vertex_info(&self) -> &'a VertexInfo<VD> {
        return &self.mesh.verts[self.vertex_index as usize];
    }
}

pub struct VertexEdgeIterator<'a, VD, ED, FD> {
    mesh: &'a Mesh<VD, ED, FD>,
    base_vertex_index: Index,
    start_edge_index: Option<Index>,
    current_edge_index: Option<Index>,
}

impl<'a, VD, ED, FD> VertexEdgeIterator<'a, VD, ED, FD> {
    pub fn vertex(&self) -> VertexRef<'a, VD, ED, FD> {
        self.mesh.vertex(self.base_vertex_index)
    }

    pub fn start_edge(&self) -> Option<EdgeRef<'a, VD, ED, FD>> {
        match self.start_edge_index {
            Some(index) => Some(EdgeRef{mesh: self.mesh, edge_index: index }),
            None => None
        }
    }
    
    fn current_edge(&self) -> Option<&EdgeInfo<ED>> {
        match self.current_edge_index {
            Some(index) => Some(&self.mesh.edges[index as usize]),
            None => None
        }
    }
}

impl<'a, VD, ED, FD> Iterator for VertexEdgeIterator<'a, VD, ED, FD> {
    type Item = EdgeRef<'a, VD, ED, FD>;

    // Iterates over the edges of a vertex.
    fn next(&mut self) -> Option<EdgeRef<'a, VD, ED, FD>> {
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

pub struct VertexFaceIterator<'a, VD, ED, FD> {
    edge_iter: VertexEdgeIterator<'a, VD, ED, FD>,
}

impl<'a, VD, ED, FD> Iterator for VertexFaceIterator<'a, VD, ED, FD> {
    type Item = FaceRef<'a, VD, ED, FD>;

    fn next(&mut self) -> Option<FaceRef<'a, VD, ED, FD>> {
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
struct HalfEdgeInfo {
    vertex_index: Index,    // required.
    next_face_index: Index, // optional. cw relative to base vertex
    next_edge_index: Index, // optional. cw around base vertex
    prev_edge_index: Index, // optional. ccw around base vertex
}

struct EdgeInfo<ED> {
    half_edge: [HalfEdgeInfo; 2],
    data: ED,
}

pub struct EdgeRef<'a, VD, ED, FD> {
    mesh: &'a Mesh<VD, ED, FD>,
    edge_index: Index 
}

impl<'a, VD, ED, FD> EdgeRef<'a, VD, ED, FD> {
    fn edge_info(&self) -> &'a EdgeInfo<ED> {
        return &self.mesh.edges[self.edge_index as usize]
    }

    pub fn data(&self) -> &'a ED {
        return &self.edge_info().data
    }

    // TODO: in non-manifold meshes, an edge might have <2 faces.
    pub fn faces(&self) -> Vec<FaceRef<'a, VD, ED, FD>> {
        let edge_info = self.edge_info();
        vec![
            FaceRef{mesh: self.mesh, face_index: edge_info.half_edge[0].next_face_index},
            FaceRef{mesh: self.mesh, face_index: edge_info.half_edge[1].next_face_index},
        ]
    }

    pub fn vertices(&self) -> Vec<VertexRef<'a, VD, ED, FD>> {
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
struct FaceInfo<FD> {
    base_edge_index: Index, // required.
    data: FD,
}

pub struct FaceRef<'a, VD, ED, FD> {
    mesh: &'a Mesh<VD, ED, FD>,
    face_index: Index,
}

impl<'a, VD, ED, FD> FaceRef<'a, VD, ED, FD> {
    fn face_info(&self) -> &'a FaceInfo<FD> {
        return &self.mesh.faces[self.face_index as usize]
    }

    pub fn data(&self) -> &FD {
        return &self.face_info().data
    }

    pub fn edge_iter(&self) -> FaceEdgeIterator<'a, VD, ED, FD> {
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
pub struct FaceEdgeIterator<'a, VD, ED, FD> {
    mesh: &'a Mesh<VD, ED, FD>,
    face_index: Index,
    start_edge_index: Index,
    current_edge_index: Index,
}

impl<'a, VD, ED, FD> Iterator for FaceEdgeIterator<'a, VD, ED, FD> {
    type Item = EdgeRef<'a, VD, ED, FD>;

    fn next(&mut self) -> Option<EdgeRef<'a, VD, ED, FD>> {
        unimplemented!(); // TODO
    }
}

#[allow(dead_code)] // TODO: remove when implemented
pub struct FaceVertexIterator<'a, VD, ED, FD> {
    edge_iter: FaceEdgeIterator<'a, VD, ED, FD>,
}

impl<'a, VD, ED, FD> Iterator for FaceVertexIterator<'a, VD, ED, FD> {
    type Item = VertexRef<'a, VD, ED, FD>;

    fn next(&mut self) -> Option<VertexRef<'a, VD, ED, FD>> {
        unimplemented!(); // TODO
    }
}

/*
 * Mesh
 */
pub struct Mesh<VD, ED, FD> {
    verts: Vec<VertexInfo<VD>>,
    edges: Vec<EdgeInfo<ED>>,
    faces: Vec<FaceInfo<FD>>,
}

impl<VD, ED, FD> Mesh<VD, ED, FD> {
    pub fn new() -> Mesh<VD, ED, FD> {
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

    pub fn vertex(&self, index: Index) -> VertexRef<VD, ED, FD> {
        return VertexRef{mesh: self, vertex_index: index}
    }
    
    pub fn edge(&self, index: Index) -> EdgeRef<VD, ED, FD> {
        return EdgeRef{mesh: self, edge_index: index}
    }

    pub fn face(&self, index: Index) -> FaceRef<VD, ED, FD> {
        return FaceRef{mesh: self, face_index: index}
    }

    pub fn vertex_iter(&self) -> MeshVertexIterator<VD, ED, FD> {
        MeshVertexIterator { mesh: self, vertex_index: 0 }
    }

    pub fn edge_iter(&self) -> MeshEdgeIterator<VD, ED, FD> {
        MeshEdgeIterator { mesh: self, edge_index: 0 }
    }

    pub fn face_iter(&self) -> MeshFaceIterator<VD, ED, FD> {
        MeshFaceIterator { mesh: self, face_index: 0 }
    }
}

pub struct MeshVertexIterator<'a, VD, ED, FD> {
    mesh: &'a Mesh<VD, ED, FD>,
    vertex_index: Index,
}

impl<'a, VD, ED, FD> Iterator for MeshVertexIterator<'a, VD, ED, FD> {
    type Item = VertexRef<'a, VD, ED, FD>;

    fn next(&mut self) -> Option<VertexRef<'a, VD, ED, FD>> {
        if (self.vertex_index as usize) < self.mesh.verts.len() {
            let ret = Some(VertexRef { mesh: self.mesh, vertex_index: self.vertex_index });
            self.vertex_index += 1;
            return ret;
        }
        return None;
    }
}

pub struct MeshEdgeIterator<'a, VD, ED, FD> {
    mesh: &'a Mesh<VD, ED, FD>,
    edge_index: Index,
}

impl<'a, VD, ED, FD> Iterator for MeshEdgeIterator<'a, VD, ED, FD> {
    type Item = EdgeRef<'a, VD, ED, FD>;

    fn next(&mut self) -> Option<EdgeRef<'a, VD, ED, FD>> {
        if (self.edge_index as usize) < self.mesh.edges.len() {
            let ret = Some(EdgeRef { mesh: self.mesh, edge_index: self.edge_index });
            self.edge_index += 1;
            return ret;
        }
        return None;
    }
}

pub struct MeshFaceIterator<'a, VD, ED, FD> {
    mesh: &'a Mesh<VD, ED, FD>,
    face_index: Index,
}

impl<'a, VD, ED, FD> Iterator for MeshFaceIterator<'a, VD, ED, FD> {
    type Item = FaceRef<'a, VD, ED, FD>;

    fn next(&mut self) -> Option<FaceRef<'a, VD, ED, FD>> {
        if (self.face_index as usize) < self.mesh.faces.len() {
            let ret = Some(FaceRef { mesh: self.mesh, face_index: self.face_index });
            self.face_index += 1;
            return ret;
        }
        return None;
    }
}
