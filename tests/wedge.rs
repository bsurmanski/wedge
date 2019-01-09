extern crate nalgebra;
extern crate wedge; 

#[cfg(test)]
mod tests {

    #[test]
    fn test_new() {
        let _l: wedge::Mesh<(), (), ()> = wedge::Mesh::new();
    }
}
