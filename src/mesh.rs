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

pub struct MutVertexRef<'a, V, E, F> {
    mesh: &'a mut Mesh<V, E, F>,
    vertex_index: Index,
}

impl<'a, V, E, F> Into<Index> for VertexRef<'a, V, E, F> {
    fn into(self) -> Index {
        return self.vertex_index;
    }
}

impl<'a, V, E, F> Into<Index> for MutVertexRef<'a, V, E, F> {
    fn into(self) -> Index {
        return self.vertex_index;
    }
}

macro_rules! impl_common_vertex_ref {
    () => {
        pub fn mesh(&self) -> &Mesh<V, E, F> {
            return self.mesh;
        }

        pub fn index(&self) -> Index {
            return self.vertex_index;
        }

        pub fn is_valid(&self) -> bool {
            self.mesh().is_valid_vertex_index(self.index())
        }

        pub fn data(&self) -> Option<&V> {
            if self.is_valid() {
                return Some(&self.vertex_info().unwrap().data);
            }
            return None;
        }

        // Private methods
        fn vertex_info(&self) -> Option<&VertexInfo<Index, V>> {
            // assume our index must exist.
            return self.mesh().vertex_info(self.index());
        }
    }
}

impl<'a, V, E, F> VertexRef<'a, V, E, F> {
    // Public methods
    pub fn new(mesh: &'a Mesh<V, E, F>, index: Index) -> Self {
        VertexRef{ mesh: mesh, vertex_index: index }
    }

    pub fn edge_iter(self) -> VertexEdgeIterator<'a, V, E, F> {
        return VertexEdgeIterator::new(self);
    }

    pub fn face_iter(self) -> VertexFaceIterator<'a, V, E, F> {
        return VertexFaceIterator {
            edge_iter: self.edge_iter()
        };
    }

    impl_common_vertex_ref!();
}

impl<'a, V, E, F> MutVertexRef<'a, V, E, F> {
    pub fn new(mesh: &'a mut Mesh<V, E, F>, index: Index) -> Self {
        MutVertexRef{ mesh: mesh, vertex_index: index }
    }

    // Private methods
    fn vertex_info_mut(&mut self) -> Option<&mut VertexInfo<Index, V>> {
        // assume our index must exist.
        return self.mesh.vertex_info_mut(self.index());
    }

    impl_common_vertex_ref!();
}

#[derive(Clone)]
pub struct VertexEdgeIterator<'a, V, E, F> {
    mesh: &'a Mesh<V, E, F>,
    base_vertex_index: Index,
    start_edge_index: Option<Index>,
    current_edge_index: Option<Index>,
}

impl<'a, V, E, F> VertexEdgeIterator<'a, V, E, F> {
    pub fn new(base: VertexRef<'a, V, E, F>) -> Self {
        if base.is_valid() {
            let base_edge = base.vertex_info().unwrap().base_edge_index.to_option();
            return VertexEdgeIterator {
                mesh: base.mesh,
                base_vertex_index: base.index(),
                start_edge_index: base_edge,
                current_edge_index: base_edge,
            };
        } else {
            return VertexEdgeIterator {
                mesh: base.mesh,
                base_vertex_index: Index::max_value(),
                start_edge_index: None,
                current_edge_index: None,
            }
        }
    }

    pub fn vertex(&self) -> VertexRef<V, E, F> {
        // assume we have a valid vertex.
        return self.mesh.vertex(self.base_vertex_index);
    }

    pub fn start_edge(&self) -> Option<EdgeRef<V, E, F>> {
        match self.start_edge_index {
            Some(index) => Some(EdgeRef{mesh: self.mesh, edge_index: index }),
            None => None
        }
    }

    fn current_edge(&self) -> Option<&EdgeInfo<E>> {
        match self.current_edge_index {
            Some(index) => self.mesh.edge_info(index),
            None => None
        }
    }
}

impl<'a, V, E, F> Iterator for VertexEdgeIterator<'a, V, E, F> {
    type Item = EdgeRef<'a, V, E, F>;

    // Iterates over the edges of a vertex.
    fn next(&mut self) -> Option<Self::Item> {
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

#[derive(Clone)]
pub struct VertexFaceIterator<'a, V, E, F> {
    edge_iter: VertexEdgeIterator<'a, V, E, F>,
}

impl<'a, V, E, F> Iterator for VertexFaceIterator<'a, V, E, F> {
    type Item = FaceRef<'a, V, E, F>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(maybe_edge) = self.edge_iter.next() {
            if let Some(edge) = maybe_edge.edge_info() {
                let face_index: Index;
                if edge.half_edge[0].vertex_index == self.edge_iter.base_vertex_index {
                    face_index = edge.half_edge[0].next_face_index;
                } else {
                    assert!(edge.half_edge[1].vertex_index == self.edge_iter.base_vertex_index,
                            "face iterator reached an face unconnected to the base vertex!");
                    face_index = edge.half_edge[1].next_face_index;
                }
                return Some(FaceRef {
                    mesh: self.edge_iter.mesh,
                    face_index: face_index,
                });
            }
        }
        return None;
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

impl HalfEdgeInfo {
    fn new() -> Self {
        HalfEdgeInfo {
            vertex_index: Index::max_value(),
            next_face_index: Index::max_value(),
            next_edge_index: Index::max_value(),
            prev_edge_index: Index::max_value(),
        }
    }
}

#[derive(Copy, Clone)]
struct EdgeInfo<E> {
    half_edge: [HalfEdgeInfo; 2],
    data: E,
}

impl<E> EdgeInfo<E> {
    fn new(data : E) -> Self {
        EdgeInfo {
            half_edge: [
                HalfEdgeInfo::new(),
                HalfEdgeInfo::new(),
            ], data: data
        }
    }

    fn next_edge_index_for_vertex(&self, base_vertex_index: Index) -> Index {
        if self.half_edge[0].vertex_index == base_vertex_index {
            return self.half_edge[0].next_edge_index;
        } else {
            assert!(self.half_edge[1].vertex_index == base_vertex_index,
                    "Attempt to call 'next_edge_for_vertex' \
                    on edge that is not connected to vertex.");
            return self.half_edge[1].next_edge_index;
        }
    }

    fn previous_edge_index_for_vertex(&self, base_vertex_index: Index) -> Index {
        if self.half_edge[0].vertex_index == base_vertex_index {
            return self.half_edge[0].next_edge_index;
        } else {
            assert!(self.half_edge[1].vertex_index == base_vertex_index,
                    "Attempt to call 'previous_edge_for_vertex' \
                    on edge that is not connected to vertex.");
            return self.half_edge[1].next_edge_index;
        }
    }

    fn half_edge_for_vertex(&self, v : Index) -> &HalfEdgeInfo {
        if self.half_edge[0].vertex_index == v {
            return &self.half_edge[0];
        }
        assert!(self.half_edge[1].vertex_index == v);
        return &self.half_edge[1];
    }

    fn half_edge_for_vertex_mut(&mut self, v : Index) -> &mut HalfEdgeInfo {
        if self.half_edge[0].vertex_index == v {
            return &mut self.half_edge[0];
        }
        assert!(self.half_edge[1].vertex_index == v);
        return &mut self.half_edge[1];
    }
}

#[derive(Copy, Clone)]
pub struct EdgeRef<'a, V, E, F> {
    mesh: &'a Mesh<V, E, F>,
    edge_index: Index
}

pub struct MutEdgeRef<'a, V, E, F> {
    mesh: &'a mut Mesh<V, E, F>,
    edge_index: Index
}

macro_rules! impl_common_edge_ref {
    () => {
        fn edge_info(&self) -> Option<&EdgeInfo<E>> {
            // assume our index must exist.
            return self.mesh.edge_info(self.edge_index);
        }

        pub fn is_valid(&self) -> bool {
            return self.mesh.is_valid_edge_index(self.edge_index);
        }

        pub fn data(&self) -> Option<&E> {
            if self.is_valid() {
                return Some(&self.edge_info().unwrap().data);
            }
            return None;
        }

        /*
        // TODO: this doesn't work for MutEdgeRef right now.
        // TODO: in non-manifold meshes, an edge might have <2 faces.
        pub fn faces(&self) -> Option<Vec<FaceRef<'a, V, E, F>>> {
            if self.is_valid() {
                let edge_info = self.edge_info().unwrap();
                return Some(vec![
                    FaceRef{mesh: self.mesh, face_index: edge_info.half_edge[0].next_face_index},
                    FaceRef{mesh: self.mesh, face_index: edge_info.half_edge[1].next_face_index},
                ]);
            }
            return None;
        }

        pub fn vertices(&self) -> Option<Vec<VertexRef<'a, V, E, F>>> {
            if self.is_valid() {
                let edge_info = self.edge_info().unwrap();
                return Some(vec![
                    VertexRef{mesh: self.mesh, vertex_index: edge_info.half_edge[0].vertex_index},
                    VertexRef{mesh: self.mesh, vertex_index: edge_info.half_edge[1].vertex_index},
                ]);
            }
            return None;
        }
        */
    }
}

impl<'a, V, E, F> EdgeRef<'a, V, E, F> {
    pub fn new(mesh: &'a Mesh<V, E, F>, index: Index) -> EdgeRef<'a, V, E, F> {
        EdgeRef { mesh: mesh, edge_index: index }
    }

    impl_common_edge_ref!();
}

impl<'a, V, E, F> MutEdgeRef<'a, V, E, F> {
    pub fn new(mesh: &'a mut Mesh<V, E, F>, index: Index) -> EdgeRef<'a, V, E, F> {
        EdgeRef { mesh: mesh, edge_index: index }
    }

    impl_common_edge_ref!();
}

/*
 * Faces
 */
#[derive(Copy, Clone)]
struct FaceInfo<F> {
    base_edge_index: Index, // required.
    data: F,
}

#[derive(Clone)]
pub struct FaceRef<'a, V, E, F> {
    mesh: &'a Mesh<V, E, F>,
    face_index: Index,
}

impl<'a, V, E, F> FaceRef<'a, V, E, F> {
    fn face_info(&self) -> &FaceInfo<F> {
        // assume our index must exist.
        return self.mesh.face_info(self.face_index).unwrap();
    }

    pub fn data(&self) -> &F {
        return &self.face_info().data
    }

    pub fn edge_iter(&self) -> FaceEdgeIterator<V, E, F> {
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
#[derive(Clone)]
pub struct FaceEdgeIterator<'a, V, E, F> {
    mesh: &'a Mesh<V, E, F>,
    face_index: Index,
    start_edge_index: Index,
    current_edge_index: Index,
}

impl<'a, V, E, F> Iterator for FaceEdgeIterator<'a, V, E, F> {
    type Item = EdgeRef<'a, V, E, F>;

    fn next(&mut self) -> Option<Self::Item> {
        unimplemented!(); // TODO
    }
}

#[allow(dead_code)] // TODO: remove when implemented
#[derive(Clone)]
pub struct FaceVertexIterator<'a, V, E, F> {
    edge_iter: FaceEdgeIterator<'a, V, E, F>,
}

impl<'a, V, E, F> Iterator for FaceVertexIterator<'a, V, E, F> {
    type Item = VertexRef<'a, V, E, F>;

    fn next(&mut self) -> Option<Self::Item> {
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
            faces: Vec::new(),
        }
    }

    pub fn is_valid_vertex_index(&self, index: Index) -> bool {
        return index < Index::max_value() && (index as usize) < self.verts.len();
    }

    pub fn is_valid_edge_index(&self, index: Index) -> bool {
        return index < Index::max_value() && (index as usize) < self.edges.len();
    }

    pub fn is_valid_face_index(&self, index: Index) -> bool {
        return index < Index::max_value() && (index as usize) < self.faces.len();
    }

    fn vertex_info(&self, index: Index) -> Option<&VertexInfo<Index, V>> {
        if self.is_valid_vertex_index(index) {
            return Some(&self.verts[index as usize]);
        }
        return None;
    }

    fn vertex_info_mut(&mut self, index: Index) -> Option<&mut VertexInfo<Index, V>> {
        if self.is_valid_vertex_index(index) {
            return Some(&mut self.verts[index as usize]);
        }
        return None;
    }

    fn edge_info(&self, index: Index) -> Option<&EdgeInfo<E>> {
        if self.is_valid_edge_index(index) {
            return Some(&self.edges[index as usize]);
        }
        return None;
    }

    fn edge_info_mut(&mut self, index: Index) -> Option<&mut EdgeInfo<E>> {
        if self.is_valid_edge_index(index) {
            return Some(&mut self.edges[index as usize]);
        }
        return None;
    }

    fn face_info(&self, index: Index) -> Option<&FaceInfo<F>> {
        if self.is_valid_face_index(index) {
            return Some(&self.faces[index as usize]);
        }
        return None;
    }

    pub fn vertex(&self, index: Index) -> VertexRef<V, E, F> {
        return VertexRef{mesh: self, vertex_index: index};
    }

    pub fn vertex_mut(&mut self, index: Index) -> MutVertexRef<V, E, F> {
        return MutVertexRef{mesh: self, vertex_index: index};
    }

    pub fn edge(&self, index: Index) -> EdgeRef<V, E, F> {
        return EdgeRef{mesh: self, edge_index: index};
    }

    pub fn face(&self, index: Index) -> FaceRef<V, E, F> {
        return FaceRef{mesh: self, face_index: index};
    }

    pub fn vertex_iter(&self) -> MeshVertexIterator<V, E, F> {
        MeshVertexIterator { mesh: self, vertex_index: 0 }
    }

    pub fn edge_iter(&self) -> MeshEdgeIterator<V, E, F> {
        MeshEdgeIterator { edge: EdgeRef::new(self, 0) }
    }

    pub fn face_iter(&self) -> MeshFaceIterator<V, E, F> {
        MeshFaceIterator { mesh: self, face_index: 0 }
    }

    pub fn add_vertex(&mut self, v: V) -> Index {
        let index = Index::new(self.verts.len());
        self.verts.push(VertexInfo::new(v));
        return index;
    }

    pub fn add_edge(&mut self, e: E, v1: Index, v2: Index) -> Index {
        let new_index = Index::new(self.edges.len());
        let mut new_edge: EdgeInfo<E> = EdgeInfo::new(e);
        let v2ref = VertexRef::new(self, v2);

        // The edge list is basically a doubley linked list.
        // Insert the new edge at the end of each edge list.
        for (i, v) in [v1, v2].iter().enumerate() {
            assert!(self.is_valid_vertex_index(*v));
            let base_edge_index = self.verts[*v as usize].base_edge_index;
            if self.is_valid_edge_index(base_edge_index) {
                let base_edge = &self.edges[base_edge_index as usize];
                let prev_edge_index = base_edge.previous_edge_index_for_vertex(*v);
                if self.is_valid_edge_index(prev_edge_index) {
                    let prev_edge = &mut self.edges[prev_edge_index as usize];
                    let mut prev_half_edge = prev_edge.half_edge_for_vertex_mut(*v);
                    prev_half_edge.next_edge_index = new_index;
                    new_edge.half_edge[i].prev_edge_index = new_index;
                }
            } else {
                self.verts[*v as usize].base_edge_index = new_index;
            }
            new_edge.half_edge[i].next_edge_index = base_edge_index;
            new_edge.half_edge[i].vertex_index = *v;
            //TODO: Deal with faces as well!!!
        }

        self.edges.push(new_edge);
        return new_index;
    }
}

#[derive(Clone)]
pub struct MeshVertexIterator<'a, V, E, F> {
    mesh: &'a Mesh<V, E, F>,
    vertex_index: Index,
}

impl<'a, V, E, F> Iterator for MeshVertexIterator<'a, V, E, F> {
    type Item = VertexRef<'a, V, E, F>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.mesh.is_valid_vertex_index(self.vertex_index) {
            let ret = Some(VertexRef {
                mesh: self.mesh,
                vertex_index: self.vertex_index
            });
            self.vertex_index += 1;
            return ret;
        }
        return None;
    }
}

#[derive(Clone)]
pub struct MeshEdgeIterator<'a, V, E, F> {
    edge: EdgeRef<'a, V, E, F>
}

impl<'a, V, E, F> Iterator for MeshEdgeIterator<'a, V, E, F> {
    type Item = EdgeRef<'a, V, E, F>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.edge.is_valid() {
            let mut tmp = EdgeRef::new(self.edge.mesh, self.edge.edge_index + 1);
            std::mem::swap(&mut self.edge, &mut tmp);
            return Some(tmp);
        }
        return None;
    }
}

#[derive(Clone)]
pub struct MeshFaceIterator<'a, V, E, F> {
    mesh: &'a Mesh<V, E, F>,
    face_index: Index,
}

impl<'a, V, E, F> Iterator for MeshFaceIterator<'a, V, E, F> {
    type Item = FaceRef<'a, V, E, F>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.mesh.is_valid_face_index(self.face_index) {
            let ret = Some(FaceRef { mesh: self.mesh, face_index: self.face_index });
            self.face_index += 1;
            return ret;
        }
        return None;
    }
}
