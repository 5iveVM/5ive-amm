import { readFile } from 'fs/promises';
import { homedir } from 'os';
import { join } from 'path';
import { Connection, Keypair, PublicKey, SystemProgram, Transaction, TransactionInstruction, } from '@solana/web3.js';
import { ACCOUNT_SIZE, TOKEN_PROGRAM_ID, createInitializeAccountInstruction, createMint, getAccount, getMinimumBalanceForRentExemptAccount, getOrCreateAssociatedTokenAccount, mintTo, } from '@solana/spl-token';
import { FiveProgram, FiveSDK } from '@5ive-tech/sdk';
const RPC_URL = process.env.FIVE_RPC_URL || 'https://api.devnet.solana.com';
const FIVE_VM_PROGRAM_ID = process.env.FIVE_VM_PROGRAM_ID || '4Qxf3pbCse2veUgZVMiAm3nWqJrYo2pT4suxHKMJdK1d';
const EXISTING_SCRIPT_ACCOUNT = process.env.FIVE_SCRIPT_ACCOUNT || '';
const CONFIRM = {
    commitment: 'confirmed',
    preflightCommitment: 'confirmed',
    skipPreflight: true,
};
const FEE_NUMERATOR = 3n;
const FEE_DENOMINATOR = 1000n;
const PROTOCOL_FEE_NUMERATOR = 1n;
function parseConsumedUnits(logs) {
    if (!logs)
        return null;
    for (const line of logs) {
        const m = line.match(/consumed (\d+) of/);
        if (m)
            return Number(m[1]);
    }
    return null;
}
function printableError(err) {
    if (err instanceof Error)
        return err.message || err.stack || err.name;
    try {
        const json = JSON.stringify(err);
        if (json && json !== '{}')
            return json;
    }
    catch {
        // ignore
    }
    return String(err);
}
async function loadPayer() {
    const path = join(homedir(), '.config/solana/id.json');
    const secret = JSON.parse(await readFile(path, 'utf8'));
    return Keypair.fromSecretKey(new Uint8Array(secret));
}
async function sendIx(connection, payer, encoded, signers, name) {
    const tx = new Transaction().add(new TransactionInstruction({
        programId: new PublicKey(encoded.programId),
        keys: encoded.keys.map((k) => ({
            pubkey: new PublicKey(k.pubkey),
            isSigner: k.isSigner,
            isWritable: k.isWritable,
        })),
        data: Buffer.from(encoded.data, 'base64'),
    }));
    tx.feePayer = payer.publicKey;
    const allSignersMap = new Map();
    allSignersMap.set(payer.publicKey.toBase58(), payer);
    for (const signer of signers)
        allSignersMap.set(signer.publicKey.toBase58(), signer);
    const requiredSignerSet = new Set(encoded.keys.filter((k) => k.isSigner).map((k) => k.pubkey));
    const neededSigners = Array.from(allSignersMap.values()).filter((kp) => kp.publicKey.equals(payer.publicKey) || requiredSignerSet.has(kp.publicKey.toBase58()));
    try {
        const signature = await connection.sendTransaction(tx, neededSigners, CONFIRM);
        const latest = await connection.getLatestBlockhash('confirmed');
        await connection.confirmTransaction({ signature, ...latest }, 'confirmed');
        const txMeta = await connection.getTransaction(signature, {
            commitment: 'confirmed',
            maxSupportedTransactionVersion: 0,
        });
        const metaErr = txMeta?.meta?.err ?? null;
        const cu = txMeta?.meta?.computeUnitsConsumed ?? parseConsumedUnits(txMeta?.meta?.logMessages);
        return {
            name,
            signature,
            computeUnits: cu,
            ok: metaErr == null,
            err: metaErr == null ? null : JSON.stringify(metaErr),
        };
    }
    catch (err) {
        return {
            name,
            signature: null,
            computeUnits: null,
            ok: false,
            err: printableError(err),
        };
    }
}
async function sendSystemTx(connection, payer, ix, signers, name) {
    try {
        const tx = new Transaction().add(ix);
        const signature = await connection.sendTransaction(tx, [payer, ...signers], CONFIRM);
        const latest = await connection.getLatestBlockhash('confirmed');
        await connection.confirmTransaction({ signature, ...latest }, 'confirmed');
        const meta = await connection.getTransaction(signature, {
            commitment: 'confirmed',
            maxSupportedTransactionVersion: 0,
        });
        return {
            name,
            signature,
            computeUnits: meta?.meta?.computeUnitsConsumed ?? parseConsumedUnits(meta?.meta?.logMessages),
            ok: meta?.meta?.err == null,
            err: meta?.meta?.err ? JSON.stringify(meta.meta.err) : null,
        };
    }
    catch (err) {
        return { name, signature: null, computeUnits: null, ok: false, err: printableError(err) };
    }
}
async function createTokenVault(connection, payer, mint, owner) {
    const vault = Keypair.generate();
    const lamports = await getMinimumBalanceForRentExemptAccount(connection);
    const tx = new Transaction().add(SystemProgram.createAccount({
        fromPubkey: payer.publicKey,
        newAccountPubkey: vault.publicKey,
        space: ACCOUNT_SIZE,
        lamports,
        programId: TOKEN_PROGRAM_ID,
    }), createInitializeAccountInstruction(vault.publicKey, mint, owner, TOKEN_PROGRAM_ID));
    const signature = await connection.sendTransaction(tx, [payer, vault], CONFIRM);
    const latest = await connection.getLatestBlockhash('confirmed');
    await connection.confirmTransaction({ signature, ...latest }, 'confirmed');
    return vault;
}
function pad(name) {
    return name.padEnd(22, ' ');
}
function makeRng(seed) {
    let s = seed >>> 0;
    return () => {
        s = (1664525 * s + 1013904223) >>> 0;
        return s;
    };
}
function buildSwapPlan() {
    const numSwaps = Number(process.env.NUM_SWAPS || '6');
    const seed = Number(process.env.RANDOM_SEED || '1337');
    if (!Number.isFinite(numSwaps) || numSwaps <= 0) {
        throw new Error(`invalid NUM_SWAPS: ${process.env.NUM_SWAPS}`);
    }
    if (numSwaps === 6 && !process.env.NUM_SWAPS) {
        return [
            { dir: 'A2B', amountIn: 300000n },
            { dir: 'A2B', amountIn: 900000n },
            { dir: 'A2B', amountIn: 1500000n },
            { dir: 'B2A', amountIn: 500000n },
            { dir: 'B2A', amountIn: 700000n },
            { dir: 'A2B', amountIn: 400000n },
        ];
    }
    const next = makeRng(seed);
    const swaps = [];
    for (let i = 0; i < numSwaps; i++) {
        const dir = (next() & 1) === 0 ? 'A2B' : 'B2A';
        const amount = 100000 + (next() % 900001); // [100k, 1,000,000]
        swaps.push({ dir, amountIn: BigInt(amount) });
    }
    return swaps;
}
function quoteOut(amountIn, reserveIn, reserveOut) {
    const protocolFee = (amountIn * PROTOCOL_FEE_NUMERATOR) / FEE_DENOMINATOR;
    const lpFee = (amountIn * (FEE_NUMERATOR - PROTOCOL_FEE_NUMERATOR)) / FEE_DENOMINATOR;
    const dxAfterFee = amountIn - protocolFee - lpFee;
    const amountOut = (reserveOut * dxAfterFee) / (reserveIn + dxAfterFee);
    const reserveInDelta = amountIn - protocolFee;
    return { protocolFee, lpFee, dxAfterFee, amountOut, reserveInDelta };
}
async function main() {
    if (!EXISTING_SCRIPT_ACCOUNT) {
        throw new Error('set FIVE_SCRIPT_ACCOUNT to an existing deployed 5ive-amm script account');
    }
    const connection = new Connection(RPC_URL, 'confirmed');
    const payer = await loadPayer();
    const artifactPath = join(process.cwd(), '..', 'build', 'main.five');
    const artifactText = await readFile(artifactPath, 'utf8');
    const loaded = await FiveSDK.loadFiveFile(artifactText);
    const program = FiveProgram.fromABI(EXISTING_SCRIPT_ACCOUNT, loaded.abi, {
        fiveVMProgramId: FIVE_VM_PROGRAM_ID,
    });
    const report = [];
    const csvRows = [];
    const decimals = 6;
    const bootstrapA = 5000000n;
    const bootstrapB = 5000000n;
    const swaps = buildSwapPlan();
    const lpUser = Keypair.generate();
    const trader = Keypair.generate();
    await sendSystemTx(connection, payer, SystemProgram.transfer({ fromPubkey: payer.publicKey, toPubkey: lpUser.publicKey, lamports: 30000000 }), [], 'setup:fund_lp_user');
    await sendSystemTx(connection, payer, SystemProgram.transfer({ fromPubkey: payer.publicKey, toPubkey: trader.publicKey, lamports: 30000000 }), [], 'setup:fund_trader');
    const mintA = await createMint(connection, payer, payer.publicKey, null, decimals);
    const mintB = await createMint(connection, payer, payer.publicKey, null, decimals);
    const lpUserAtaA = await getOrCreateAssociatedTokenAccount(connection, payer, mintA, lpUser.publicKey);
    const lpUserAtaB = await getOrCreateAssociatedTokenAccount(connection, payer, mintB, lpUser.publicKey);
    const traderAtaA = await getOrCreateAssociatedTokenAccount(connection, payer, mintA, trader.publicKey);
    const traderAtaB = await getOrCreateAssociatedTokenAccount(connection, payer, mintB, trader.publicKey);
    await mintTo(connection, payer, mintA, lpUserAtaA.address, payer, Number(bootstrapA * 10n));
    await mintTo(connection, payer, mintB, lpUserAtaB.address, payer, Number(bootstrapB * 10n));
    await mintTo(connection, payer, mintA, traderAtaA.address, payer, Number(bootstrapA * 10n));
    await mintTo(connection, payer, mintB, traderAtaB.address, payer, Number(bootstrapB * 10n));
    const pool = Keypair.generate();
    const vaultA = await createTokenVault(connection, payer, mintA, pool.publicKey);
    const vaultB = await createTokenVault(connection, payer, mintB, pool.publicKey);
    const lpMint = await createMint(connection, payer, pool.publicKey, null, decimals);
    const lpUserLpAta = await getOrCreateAssociatedTokenAccount(connection, payer, lpMint, lpUser.publicKey);
    const initIx = (await program
        .function('init_pool')
        .payer(payer.publicKey.toBase58())
        .accounts({ pool: pool.publicKey.toBase58(), creator: payer.publicKey.toBase58() })
        .args({
        token_a_mint: mintA.toBase58(),
        token_b_mint: mintB.toBase58(),
        token_a_vault: vaultA.publicKey.toBase58(),
        token_b_vault: vaultB.publicKey.toBase58(),
        lp_mint: lpMint.toBase58(),
        fee_numerator: Number(FEE_NUMERATOR),
        fee_denominator: Number(FEE_DENOMINATOR),
        protocol_fee_numerator: Number(PROTOCOL_FEE_NUMERATOR),
    })
        .instruction());
    report.push(await sendIx(connection, payer, initIx, [pool], 'init_pool'));
    const bootstrapIx = (await program
        .function('bootstrap_liquidity')
        .payer(payer.publicKey.toBase58())
        .accounts({
        pool: pool.publicKey.toBase58(),
        user_token_a: lpUserAtaA.address.toBase58(),
        user_token_b: lpUserAtaB.address.toBase58(),
        pool_token_a_vault: vaultA.publicKey.toBase58(),
        pool_token_b_vault: vaultB.publicKey.toBase58(),
        lp_mint: lpMint.toBase58(),
        user_lp_account: lpUserLpAta.address.toBase58(),
        user_authority: lpUser.publicKey.toBase58(),
        token_program: TOKEN_PROGRAM_ID.toBase58(),
    })
        .args({
        amount_a: Number(bootstrapA),
        amount_b: Number(bootstrapB),
        min_liquidity: Number(bootstrapA + bootstrapB),
    })
        .instruction());
    report.push(await sendIx(connection, payer, bootstrapIx, [lpUser, pool], 'bootstrap'));
    if (!report[0].ok || !report[1].ok) {
        throw new Error(`setup failed: ${JSON.stringify(report, null, 2)}`);
    }
    let vaultBalanceA = (await getAccount(connection, vaultA.publicKey, 'confirmed', TOKEN_PROGRAM_ID)).amount;
    let vaultBalanceB = (await getAccount(connection, vaultB.publicKey, 'confirmed', TOKEN_PROGRAM_ID)).amount;
    let protocolFeesA = 0n;
    let protocolFeesB = 0n;
    let reserveA = vaultBalanceA - protocolFeesA;
    let reserveB = vaultBalanceB - protocolFeesB;
    console.log('--- multi swap curve verification ---');
    console.log('rpc:', RPC_URL);
    console.log('five_vm_program_id:', FIVE_VM_PROGRAM_ID);
    console.log('script_account:', EXISTING_SCRIPT_ACCOUNT);
    console.log('pool:', pool.publicKey.toBase58());
    console.log('vault_a:', vaultA.publicKey.toBase58());
    console.log('vault_b:', vaultB.publicKey.toBase58());
    console.log('initial_vault_a:', vaultBalanceA.toString());
    console.log('initial_vault_b:', vaultBalanceB.toString());
    console.log('initial_effective_reserve_a:', reserveA.toString());
    console.log('initial_effective_reserve_b:', reserveB.toString());
    for (let i = 0; i < swaps.length; i++) {
        const s = swaps[i];
        const isAToB = s.dir === 'A2B';
        const reserveInBefore = isAToB ? reserveA : reserveB;
        const reserveOutBefore = isAToB ? reserveB : reserveA;
        const kBefore = reserveA * reserveB;
        const quote = quoteOut(s.amountIn, reserveInBefore, reserveOutBefore);
        if (quote.amountOut <= 0n) {
            throw new Error(`swap ${i + 1} quote returned zero`);
        }
        const srcAta = isAToB ? traderAtaA.address : traderAtaB.address;
        const dstAta = isAToB ? traderAtaB.address : traderAtaA.address;
        const srcVault = isAToB ? vaultA.publicKey : vaultB.publicKey;
        const dstVault = isAToB ? vaultB.publicKey : vaultA.publicKey;
        const traderSrcBefore = (await getAccount(connection, srcAta, 'confirmed', TOKEN_PROGRAM_ID)).amount;
        const traderDstBefore = (await getAccount(connection, dstAta, 'confirmed', TOKEN_PROGRAM_ID)).amount;
        const ix = (await program
            .function('swap')
            .payer(payer.publicKey.toBase58())
            .accounts({
            pool: pool.publicKey.toBase58(),
            user_source: srcAta.toBase58(),
            user_destination: dstAta.toBase58(),
            pool_source_vault: srcVault.toBase58(),
            pool_destination_vault: dstVault.toBase58(),
            user_authority: trader.publicKey.toBase58(),
            token_program: TOKEN_PROGRAM_ID.toBase58(),
        })
            .args({
            amount_in: Number(s.amountIn),
            min_amount_out: Number(quote.amountOut - 1n),
            is_a_to_b: isAToB,
        })
            .instruction());
        const step = await sendIx(connection, payer, ix, [trader, pool], `swap_${i + 1}_${s.dir}`);
        report.push(step);
        const traderSrcAfter = (await getAccount(connection, srcAta, 'confirmed', TOKEN_PROGRAM_ID)).amount;
        const traderDstAfter = (await getAccount(connection, dstAta, 'confirmed', TOKEN_PROGRAM_ID)).amount;
        const vaultAAfter = (await getAccount(connection, vaultA.publicKey, 'confirmed', TOKEN_PROGRAM_ID)).amount;
        const vaultBAfter = (await getAccount(connection, vaultB.publicKey, 'confirmed', TOKEN_PROGRAM_ID)).amount;
        const srcDelta = traderSrcAfter - traderSrcBefore;
        const dstDelta = traderDstAfter - traderDstBefore;
        const actualOut = dstDelta;
        const actualIn = -srcDelta;
        if (!step.ok) {
            throw new Error(`swap ${i + 1} failed: ${step.err}`);
        }
        if (actualIn !== s.amountIn) {
            throw new Error(`swap ${i + 1}: input mismatch expected=${s.amountIn} actual=${actualIn}`);
        }
        if (actualOut !== quote.amountOut) {
            throw new Error(`swap ${i + 1}: output mismatch expected=${quote.amountOut} actual=${actualOut}`);
        }
        if (isAToB) {
            protocolFeesA += quote.protocolFee;
        }
        else {
            protocolFeesB += quote.protocolFee;
        }
        vaultBalanceA = vaultAAfter;
        vaultBalanceB = vaultBAfter;
        reserveA = vaultBalanceA - protocolFeesA;
        reserveB = vaultBalanceB - protocolFeesB;
        const kAfter = reserveA * reserveB;
        const expectedReserveIn = reserveInBefore + quote.reserveInDelta;
        const expectedReserveOut = reserveOutBefore - quote.amountOut;
        const actualReserveIn = isAToB ? reserveA : reserveB;
        const actualReserveOut = isAToB ? reserveB : reserveA;
        if (actualReserveIn !== expectedReserveIn || actualReserveOut !== expectedReserveOut) {
            throw new Error(`swap ${i + 1}: reserve mismatch expected_in=${expectedReserveIn} actual_in=${actualReserveIn} expected_out=${expectedReserveOut} actual_out=${actualReserveOut}`);
        }
        if (kAfter < kBefore) {
            throw new Error(`swap ${i + 1}: invariant decreased k_before=${kBefore} k_after=${kAfter}`);
        }
        console.log(`${pad(step.name)} | sig=${step.signature} | cu=${step.computeUnits ?? 'n/a'} | in=${s.amountIn} | out=${quote.amountOut} | k_before=${kBefore} | k_after=${kAfter} | fee_a=${protocolFeesA} | fee_b=${protocolFeesB}`);
        csvRows.push({
            idx: i + 1,
            dir: s.dir,
            amountIn: s.amountIn,
            amountOut: quote.amountOut,
            signature: step.signature ?? '',
            cu: step.computeUnits,
            kBefore,
            kAfter,
        });
    }
    console.log('\nsummary:');
    for (const item of report) {
        if (item.name === 'init_pool' || item.name === 'bootstrap' || item.name.startsWith('swap_')) {
            console.log(`${pad(item.name)} | ok=${item.ok} | sig=${item.signature ?? 'n/a'} | cu=${item.computeUnits ?? 'n/a'} | err=${item.err ?? 'none'}`);
        }
    }
    console.log('\nBEGIN_CSV');
    console.log('idx,dir,amount_in,amount_out,compute_units,k_before,k_after,signature');
    for (const row of csvRows) {
        console.log(`${row.idx},${row.dir},${row.amountIn},${row.amountOut},${row.cu ?? ''},${row.kBefore},${row.kAfter},${row.signature}`);
    }
    console.log('END_CSV');
}
main().catch((err) => {
    console.error('run failed:', printableError(err));
    process.exit(1);
});
