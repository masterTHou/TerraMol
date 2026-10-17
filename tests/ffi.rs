use mol_engine_core::ffi::*;
#[test]
fn lifecycle_and_buffers() {
    unsafe {
        assert_eq!(me_create(100., 1., std::ptr::null_mut()), 1);
        let mut h = 0;
        assert_eq!(me_create(f64::NAN, 1., &mut h), 1);
        assert_eq!(h, 0);
        assert_eq!(me_create(100., 1., &mut h), 0);
        let mut id = 0;
        assert_eq!(me_spawn(h, 0, 50., 50., 50., &mut id), 0);
        let mut n = 0;
        assert_eq!(me_snapshot_size(h, &mut n), 0);
        assert_eq!(n, 40 + 3 * 40 + 2 * 16);
        let mut required = 0;
        assert_eq!(
            me_snapshot_copy(h, std::ptr::null_mut(), 0, &mut required),
            3
        );
        assert_eq!(required, n);
        let mut error = [0u8; 256];
        assert_eq!(me_last_error(error.as_mut_ptr(), 256, &mut required), 0);
        assert!(std::str::from_utf8(&error).unwrap().contains("too small"));
        assert_eq!(
            me_snapshot_copy(h, std::ptr::null_mut(), n, &mut required),
            1
        );
        let mut b = vec![0; n as usize];
        assert_eq!(me_snapshot_copy(h, b.as_mut_ptr(), n, &mut required), 0);
        assert_eq!(u32::from_le_bytes(b[8..12].try_into().unwrap()), 3);
        assert_eq!(u64::from_le_bytes(b[72..80].try_into().unwrap()), id);
        assert_eq!(me_set_velocity(h, id, f64::INFINITY, 0., 0.), 1);
        assert_eq!(me_destroy(h), 0);
        assert_eq!(me_step(h, 1), 2);
        assert_eq!(me_destroy(h), 2);
    }
}
#[test]
fn parallel_handles() {
    let threads: Vec<_> = (0..8)
        .map(|_| {
            std::thread::spawn(|| unsafe {
                let mut h = 0;
                assert_eq!(me_create(100., 1., &mut h), 0);
                assert_eq!(me_batch_spawn(h, 1, 50, 9, 100), 0);
                assert_eq!(me_step(h, 10), 0);
                assert_eq!(me_destroy(h), 0);
                h
            })
        })
        .collect();
    let mut ids: Vec<_> = threads.into_iter().map(|t| t.join().unwrap()).collect();
    ids.sort();
    ids.dedup();
    assert_eq!(ids.len(), 8);
}
