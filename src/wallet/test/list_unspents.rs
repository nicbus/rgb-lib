use super::*;

#[test]
fn success() {
    initialize();

    let amount: u64 = 66;

    // wallets
    let (mut wallet, online) = get_empty_wallet!();
    let (mut rcv_wallet, rcv_online) = get_funded_wallet!();

    // no unspents
    let unspent_list_settled = wallet.list_unspents(true).unwrap();
    assert_eq!(unspent_list_settled.len(), 0);
    let unspent_list_all = wallet.list_unspents(false).unwrap();
    assert_eq!(unspent_list_all.len(), 0);

    fund_wallet(wallet.get_address());
    mine(false);

    // one (settled) unspent, no RGB allocations
    wallet._sync_db_txos().unwrap();
    let unspent_list_settled = wallet.list_unspents(true).unwrap();
    assert_eq!(unspent_list_settled.len(), 1);
    let unspent_list_all = wallet.list_unspents(false).unwrap();
    assert_eq!(unspent_list_all.len(), 1);

    test_create_utxos_default(&mut wallet, online.clone());

    // multiple unspents, one settled RGB allocation
    let asset = wallet
        .issue_asset_rgb20(
            online.clone(),
            TICKER.to_string(),
            NAME.to_string(),
            PRECISION,
            vec![AMOUNT],
        )
        .unwrap();
    let unspent_list_settled = wallet.list_unspents(true).unwrap();
    assert_eq!(unspent_list_settled.len(), UTXO_NUM as usize + 1);
    let unspent_list_all = wallet.list_unspents(false).unwrap();
    assert_eq!(unspent_list_all.len(), UTXO_NUM as usize + 1);
    let mut settled_allocations = vec![];
    unspent_list_settled
        .iter()
        .for_each(|u| settled_allocations.extend(u.rgb_allocations.clone()));
    assert_eq!(settled_allocations.len(), 1);
    assert!(settled_allocations
        .iter()
        .all(|a| a.asset_id == Some(asset.asset_id.clone()) && a.amount == AMOUNT && a.settled));

    // multiple unspents, one failed blind, not listed
    let blind_data_fail = rcv_wallet
        .blind(None, None, None, CONSIGNMENT_ENDPOINTS.clone())
        .unwrap();
    rcv_wallet
        .fail_transfers(
            rcv_online.clone(),
            Some(blind_data_fail.blinded_utxo),
            None,
            false,
        )
        .unwrap();
    show_unspent_colorings(&rcv_wallet, "after blind fail");
    let unspent_list_all = rcv_wallet.list_unspents(false).unwrap();
    let mut allocations = vec![];
    unspent_list_all
        .iter()
        .for_each(|u| allocations.extend(u.rgb_allocations.clone()));
    assert_eq!(allocations.len(), 0);
    // one failed send, not listed
    let blind_data = rcv_wallet
        .blind(None, None, None, CONSIGNMENT_ENDPOINTS.clone())
        .unwrap();
    let recipient_map = HashMap::from([(
        asset.asset_id.clone(),
        vec![Recipient {
            amount,
            blinded_utxo: blind_data.blinded_utxo,
            consignment_endpoints: CONSIGNMENT_ENDPOINTS.clone(),
        }],
    )]);
    let txid = wallet.send(online.clone(), recipient_map, false).unwrap();
    assert!(!txid.is_empty());
    wallet
        .fail_transfers(online.clone(), None, Some(txid), false)
        .unwrap();
    show_unspent_colorings(&wallet, "after send fail");
    let unspent_list_all = wallet.list_unspents(false).unwrap();
    dbg!(&unspent_list_all);
    let mut allocations = vec![];
    unspent_list_all
        .iter()
        .for_each(|u| allocations.extend(u.rgb_allocations.clone()));
    assert_eq!(allocations.len(), 1);
    assert!(allocations
        .iter()
        .all(|a| a.asset_id == Some(asset.asset_id.clone()) && a.amount == AMOUNT && a.settled));

    drain_wallet(&wallet, online.clone());
    fund_wallet(wallet.get_address());
    mine(false);
    test_create_utxos_default(&mut wallet, online.clone());
    drain_wallet(&rcv_wallet, rcv_online.clone());
    fund_wallet(rcv_wallet.get_address());
    mine(false);
    test_create_utxos_default(&mut rcv_wallet, rcv_online.clone());

    // issue + send some asset
    let asset = wallet
        .issue_asset_rgb20(
            online.clone(),
            TICKER.to_string(),
            NAME.to_string(),
            PRECISION,
            vec![AMOUNT],
        )
        .unwrap();
    let blind_data = rcv_wallet
        .blind(None, None, None, CONSIGNMENT_ENDPOINTS.clone())
        .unwrap();
    let recipient_map = HashMap::from([(
        asset.asset_id.clone(),
        vec![Recipient {
            amount,
            blinded_utxo: blind_data.blinded_utxo,
            consignment_endpoints: CONSIGNMENT_ENDPOINTS.clone(),
        }],
    )]);
    let txid = wallet.send(online.clone(), recipient_map, false).unwrap();
    assert!(!txid.is_empty());
    show_unspent_colorings(&rcv_wallet, "receiver after send - WaitingCounterparty");
    show_unspent_colorings(&wallet, "sender after send - WaitingCounterparty");
    // check receiver lists no settled allocations
    let rcv_unspent_list = rcv_wallet.list_unspents(true).unwrap();
    assert!(!rcv_unspent_list
        .iter()
        .any(|u| !u.rgb_allocations.is_empty()));
    // check receiver lists one pending blind
    let rcv_unspent_list_all = rcv_wallet.list_unspents(false).unwrap();
    dbg!(&rcv_unspent_list_all);
    let mut allocations = vec![];
    rcv_unspent_list_all
        .iter()
        .for_each(|u| allocations.extend(u.rgb_allocations.clone()));
    assert!(!allocations.iter().any(|a| a.settled));
    assert_eq!(allocations.iter().filter(|a| !a.settled).count(), 1);
    // check sender lists one settled issue
    let unspent_list_settled = wallet.list_unspents(true).unwrap();
    let mut settled_allocations = vec![];
    unspent_list_settled
        .iter()
        .for_each(|u| settled_allocations.extend(u.rgb_allocations.clone()));
    assert_eq!(settled_allocations.len(), 1);
    assert!(settled_allocations
        .iter()
        .all(|a| a.asset_id == Some(asset.asset_id.clone()) && a.amount == AMOUNT && a.settled));
    // check sender lists one pending change
    let unspent_list_all = wallet.list_unspents(false).unwrap();
    dbg!(&unspent_list_all);
    let mut pending_allocations = vec![];
    unspent_list_all
        .iter()
        .for_each(|u| pending_allocations.extend(u.rgb_allocations.iter().filter(|a| !a.settled)));
    assert_eq!(pending_allocations.len(), 1);
    assert!(pending_allocations
        .iter()
        .all(|a| a.asset_id == Some(asset.asset_id.clone()) && a.amount == AMOUNT - amount));

    stop_mining();

    // transfer progresses to status WaitingConfirmations
    rcv_wallet
        .refresh(rcv_online.clone(), None, vec![])
        .unwrap();
    wallet
        .refresh(online.clone(), Some(asset.asset_id.clone()), vec![])
        .unwrap();
    show_unspent_colorings(&rcv_wallet, "receiver after send - WaitingConfirmations");
    show_unspent_colorings(&wallet, "sender after send - WaitingConfirmations");
    // check receiver lists no settled allocations
    let rcv_unspent_list = rcv_wallet.list_unspents(true).unwrap();
    assert!(!rcv_unspent_list
        .iter()
        .any(|u| !u.rgb_allocations.is_empty()));
    // check receiver lists one pending blind
    let rcv_unspent_list_all = rcv_wallet.list_unspents(false).unwrap();
    dbg!(&rcv_unspent_list_all);
    let mut allocations = vec![];
    rcv_unspent_list_all
        .iter()
        .for_each(|u| allocations.extend(u.rgb_allocations.clone()));
    assert!(!allocations.iter().any(|a| a.settled));
    assert_eq!(allocations.iter().filter(|a| !a.settled).count(), 1);
    assert!(allocations
        .iter()
        .all(|a| a.asset_id == Some(asset.asset_id.clone()) && a.amount == amount));
    // check sender lists one settled issue
    let unspent_list_settled = wallet.list_unspents(true).unwrap();
    let mut settled_allocations = vec![];
    unspent_list_settled
        .iter()
        .for_each(|u| settled_allocations.extend(u.rgb_allocations.clone()));
    assert_eq!(settled_allocations.len(), 1);
    assert!(settled_allocations
        .iter()
        .all(|a| a.asset_id == Some(asset.asset_id.clone()) && a.amount == AMOUNT && a.settled));
    // check sender lists one pending change
    let unspent_list_all = wallet.list_unspents(false).unwrap();
    dbg!(&unspent_list_all);
    let mut pending_allocations = vec![];
    unspent_list_all
        .iter()
        .for_each(|u| pending_allocations.extend(u.rgb_allocations.iter().filter(|a| !a.settled)));
    assert_eq!(pending_allocations.len(), 1);
    assert!(pending_allocations
        .iter()
        .all(|a| a.asset_id == Some(asset.asset_id.clone()) && a.amount == AMOUNT - amount));

    // transfer progresses to status Settled
    mine(true);
    rcv_wallet.refresh(rcv_online, None, vec![]).unwrap();
    wallet
        .refresh(online, Some(asset.asset_id.clone()), vec![])
        .unwrap();
    show_unspent_colorings(&rcv_wallet, "receiver after send - Settled");
    show_unspent_colorings(&wallet, "sender after send - Settled");
    // check receiver lists one settled allocation
    let rcv_unspent_list = rcv_wallet.list_unspents(true).unwrap();
    let mut settled_allocations = vec![];
    rcv_unspent_list
        .iter()
        .for_each(|u| settled_allocations.extend(u.rgb_allocations.clone()));
    assert!(settled_allocations.iter().all(|a| a.settled));
    assert_eq!(settled_allocations.len(), 1);
    assert!(settled_allocations
        .iter()
        .all(|a| a.asset_id == Some(asset.asset_id.clone()) && a.amount == amount));
    // check receiver lists no pending allocations
    let rcv_unspent_list_all = rcv_wallet.list_unspents(false).unwrap();
    dbg!(&rcv_unspent_list_all);
    let mut allocations = vec![];
    rcv_unspent_list_all
        .iter()
        .for_each(|u| allocations.extend(u.rgb_allocations.clone()));
    assert_eq!(allocations, settled_allocations);
    // check sender lists one settled change
    let unspent_list_settled = wallet.list_unspents(true).unwrap();
    let mut settled_allocations = vec![];
    unspent_list_settled
        .iter()
        .for_each(|u| settled_allocations.extend(u.rgb_allocations.clone()));
    assert_eq!(settled_allocations.len(), 1);
    assert!(settled_allocations
        .iter()
        .all(|a| a.asset_id == Some(asset.asset_id.clone())
            && a.amount == AMOUNT - amount
            && a.settled));
    // check sender lists no pending allocations
    let unspent_list_all = wallet.list_unspents(false).unwrap();
    dbg!(&unspent_list_all);
    let mut allocations = vec![];
    unspent_list_all
        .iter()
        .for_each(|u| allocations.extend(u.rgb_allocations.clone()));
    assert_eq!(allocations, settled_allocations);
}
