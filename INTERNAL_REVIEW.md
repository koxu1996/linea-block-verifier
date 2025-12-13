# Notes from Internal Review

Analysis of prototype implementation.

## TL;DR

1. Get beacon_block_root and its signature from untrusted BeaconBlock - verify if it is valid, signed by Linea. At this point beacon_block_root is known as valid and signed by linea.
2. Get untrusted BeaconHeader and compute root hash - cross check it against beacon_block_root from step 1. At this point BeaconHeader is trusted (linked with beacon_block_root).
3. Using untrusted ExecutionPayload from untrusted BeaconBlock, compute body hash and compare it to body in the (already) trusted BeaconHeader. At this point we trust ExecutionPayload (linked with trusted BeaconHeader
4. Run execution block hashing, and compare it with hash in exeuction payload. That's it! 

## 1. What is signed by Linea?

Linea sequencer signs some hash called 'beacon_block_root'.

WHERE IS IT?
Both message - beacon block root hash - and its signature is located
inside BlockResponse (data fetched from Beacon API), under
`.data.message.body.attestations` (first signature, for current slot).

VERIFICATION?
We verify 'signature' by calling `recover_address_from_prehash()`,
then comparing signer address. This cannot be faked.

WHAT WE LEARN?
We know for sure that 'some' beacon block root hash was signed by Linea,
that is all. What it implies? For that we need to understand how beacon 
block root hash is computed!

CHECK NUMBER?
6

## 2. How `beacon_block_root` is computed?

It is done by hashing 'beacon block header', we cross-check it against
the provided (Linea signed) one.

WHERE IS INPUT?
Beacon block header is taken from `HeaderResponse`, field `.data.message`.
Target hash (for cross-check) is taken from the part of `BlockResponse` at field
`.data.message.body.attestations`.

HOW HASHING IS DONE?
Keccak of RLP-encoded BeaconHeader. *All* fields are encoded as:
`[slot, round, timestamp, [proposer_index], parent_root, state_root, body_root]`.

VERIFICATION?
Hashing cannot be faked, so we get a real root hash of BeaconHeader data.
Then we cross-check it against the hash signed by Linea - if this matches then
we basically linked *full BeaconHeader data* with correct Linea signature.

WHAT WE LEARN?
We know the *exact BeaconHeader data* that was hashed and correctly signed by Linea.
Now using trusted *.body_root* we can validate *BeaconBlock* data. 

CHECK NUMBER?
4

At this point we get 'some' BeaconHeader, that was correctly signed by Linea.
We know body_root (hash of body data)

## 3. How `body_root` is computed?

Body hash is computed based on `ExecutionPayload` inside `BlockResponse`,
using previous block attestation (also got from BlockResponse, so untrusted).

WHERE IS INPUT?
ExecutionPayload is taken from `BlockResponse`, field `.data.message.body.execution_payload`.
We get previous block attestation from `BlockResponse` too, part of the field
`.data.message.body.attestations`.

HOW HASHING IS DONE?
Keccak of RLP-encoded ExecutionPayload. *All* fields are encoded as:
`[[attestation_sigs], ep.parent_hash, ep.fee_recipient, ep.state_root, ep.receipts_root,
ep.logs_bloom, ep.prev_randao, ep.block_number, ep.gas_limit, ep.gas_used, ep.timestamp,
ep.extra_data, ep.base_fee_per_gas, ep.block_hash, ep.transactions]`,
where `ep` is execution payload.

VERIFCATION?
Hashing cannot be faked, so we get a real hash of ExecutionPayload data.
Then we can cross-check it against `.body_root` used in trusted header - if this matches
then we basically linked *full ExecutionPayload data* with correct Linea signature.

WHAT WE LEARN?
We know the *exact ExecutionPayload data* that was hashed and correctly signed by Linea.

CHECK NUMBER?
5

At this point we confirmed ExecutionPayload data (part of the beacon block) that got
signed as part of the (already) trusted 'some' header by Linea.

## 4. What are the extra checks for `BlockResponse`?

To confirm other - non ExecutionPayload - parts of `BlockResponse`, we additionally check:
- .data.signature - must be empty in case of Linea (no BLS used yet) - check 1
- .data.message.proposer_index - must be correct Linea address - check 2.

NOTE: Attestations - `.data.message.body.attestations` - are NOT checked per se, but we use them
to find the expected beacon block root for current slot (.data.message.slot). This does NOT have to
be checked, because we already confirmed slot + beacon root hash (just by computing beacon header data).
So even though attestations are not validated directly, putting wrong or fake values would not work, 
because of the BeaconHeader validation (only Linea could fake it). Potentially someone could put more
attestations in the BlockResponse (we are not checking if there are only 2), but it would still fail due to header validation.
TL;DR: We are safe!

At this point we confirmed validity of all the data in BeaconBlock (*see below note about attestations*).
Potentially attestation could have more items that are unused, but that does NOT affect security - they are
just not used. This could be a topic to discuss - RPC sanity checks.

