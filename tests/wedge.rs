extern crate wedge; 

#[cfg(test)]
mod tests {

    #[test]
    fn test_new() {
        let _l: wedge::mesh::Mesh<(), (), ()> = wedge::mesh::Mesh::new();
    }

    #[test]
    fn test_add_verts() {
        let mut mesh: wedge::mesh::Mesh<(u32), (), ()> = wedge::mesh::Mesh::new();
        mesh.add_vertex(5);
        mesh.add_vertex(11);
        mesh.add_vertex(15);
        assert_eq!(*mesh.vertex(0).data(), 5);
        assert_eq!(*mesh.vertex(1).data(), 11);
        assert_eq!(*mesh.vertex(2).data(), 15);
    }

    #[test]
    fn test_mesh_vert_iterator() {
        let mut mesh: wedge::mesh::Mesh<(u32), (), ()> = wedge::mesh::Mesh::new();
        mesh.add_vertex(5);
        mesh.add_vertex(11);
        mesh.add_vertex(15);
        let expect = vec![5, 11, 15];
        let mut i = 0;
        for v in mesh.vertex_iter() {
            assert_eq!(*v.data(), expect[i]);
            i += 1; 
        }
        assert_eq!(i, 3);
    }
}
