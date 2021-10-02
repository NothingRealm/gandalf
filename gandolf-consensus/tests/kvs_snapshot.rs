mod fixtures;

use gandolf_consensus::raft::State;

use tokio::time::Duration;

use fixtures::kvs_helpers::{client_write_requset, kvs_cluster_of_nth};

#[tokio::test(flavor = "multi_thread", worker_threads = 5)]
async fn test_snapshot() -> gandolf_consensus::Result<()> {
    let cluster = kvs_cluster_of_nth(5).await?;

    let mut node1 = cluster.get(0).unwrap().0.borrow_mut();
    let mut node2 = cluster.get(1).unwrap().0.borrow_mut();
    let mut node3 = cluster.get(2).unwrap().0.borrow_mut();
    let mut node4 = cluster.get(3).unwrap().0.borrow_mut();
    let mut node5 = cluster.get(4).unwrap().0.borrow_mut();

    node1.current_term = 1;
    node1.set_state(State::Leader);

    node2.current_term = 1;
    node2.current_leader = Some(node1.id.clone());

    node3.current_term = 1;
    node3.current_leader = Some(node1.id.clone());

    node4.current_term = 1;
    node4.current_leader = Some(node1.id.clone());

    node5.current_term = 1;
    node5.current_leader = Some(node1.id.clone());

    let connection_addr = format!("127.0.0.1:{}", 9876).to_string();

    tokio::select! {
        _ = node1.run() => {
            assert!(false);
        },
        _ = node2.run()  => {
            assert!(false);
        },
        _ = node3.run()  => {
            assert!(false);
        },
        _ = node4.run()  => {
            assert!(false);
        },
        _ = node5.run()  => {
            assert!(false);
        },
        res = client_write_requset(200, connection_addr, Duration::from_secs(0)) => {
            res?
        }
    }

    drop(node1);
    drop(node2);
    drop(node3);
    drop(node4);
    drop(node5);

    let _: Vec<_> = cluster.
        iter()
        .map(|c| assert!(c.0.borrow().snapshot_num == 2)).collect();

    Ok(())
}
