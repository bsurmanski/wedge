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
        assert_eq!(*mesh.vertex(0).data().unwrap(), 5);
        assert_eq!(*mesh.vertex(1).data().unwrap(), 11);
        assert_eq!(*mesh.vertex(2).data().unwrap(), 15);
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
            assert_eq!(*v.data().unwrap(), expect[i]);
            i += 1; 
        }
        assert_eq!(i, 3);
    }

    #[test]
    fn test_add_basic_edge() {
        let mut mesh: wedge::mesh::Mesh<(u32), (f32), ()> = wedge::mesh::Mesh::new();
        let v1 = mesh.add_vertex(5);
        let v2 = mesh.add_vertex(11);
        let v3 = mesh.add_vertex(15);
        let v4 = mesh.add_vertex(22);
        mesh.add_edge(5.5, v1, v2);
        mesh.add_edge(3.1, v2, v3);
        mesh.add_edge(2.2, v3, v1);
        mesh.add_edge(1.1, v1, v4);
        let expect = vec![5.5, 3.1, 2.2, 1.1];
        let mut i = 0;
        for e in mesh.edge_iter() {
            assert_eq!(*e.data().unwrap(), expect[i]);
            i += 1;
        }
        assert_eq!(i, 4);

        // Check that v1 has the right edges.
        let expect_2 = vec![0, 2, 3];
        i = 0;
        for e in mesh.vertex(v1).edge_iter() {
            assert_eq!(e.index(), expect_2[i]);
        }
    }
}
