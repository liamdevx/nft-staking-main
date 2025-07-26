// app/job-cli.ts
import { Command } from 'commander';
import { adminKeyPair, backendWalletKeypair, mint, programId, tokenAddress, userKeyPair } from "./Constants";
import { Keypair, PublicKey } from "@solana/web3.js";
import { Program } from '@coral-xyz/anchor';
import { BN } from 'bn.js';
// import { setUserAllocation } from './setUSerAllocation';
import { mintTo } from '@solana/spl-token';
import { NftStaking } from '../target/types/nft_staking';
import { loadProgram } from './utils';
import { initProgram } from './initProgram';
import { stakeNft } from './stakeNft';
import { unstakeNft } from './unstakeNft';
import { addCollection } from './addCollection';
import { addReward } from './addRewards';
import { claimRewards } from './claimRewards';

const program = new Command();

const debug = require('debug')('job:main');
function programCommand(
    name: string,
    options: { requireWallet: boolean } = { requireWallet: true },
) {
    let cmProgram = program
        .command(name)
        .option(
            '-e, --env <string>',
            'Solana cluster env name',
            'devnet', //mainnet-beta, testnet, devnet
        )
        .option('-r, --rpc <string>', 'rpc endpoint', 'https://api.devnet.solana.com');

    // if (options.requireWallet) {
    //     cmProgram = cmProgram.requiredOption('-k, --keypair <path>', `Solana wallet location`);
    // }

    return cmProgram;
}

programCommand('initProgram')
    .action(async (options) => {
        debug('options: ', options);
        const keypairPath = options.keypair;
        const env = options.env;
        const rpc = options.rpc;
        debug('keypairPath: ', keypairPath);
        const adminWallet = Keypair.fromSecretKey(new Uint8Array(adminKeyPair));
        console.log('adminWallet: ', adminWallet.publicKey.toString());
        //devnet rpc: https://winter-flashy-dawn.solana-devnet.discover.quiknode.pro/c7023a9cfda5932a4e18ec7f381e98cc2226c22e/
        const program: Program<NftStaking> = await loadProgram(adminWallet, env, programId.toString(), rpc);
        await initProgram(program, adminWallet);
    });

programCommand('addCollection')
.action(async (options) => {
    debug('options: ', options);
    const keypairPath = options.keypair;
    const env = options.env;
    const rpc = options.rpc;
    debug('keypairPath: ', keypairPath);
    const adminWallet = Keypair.fromSecretKey(new Uint8Array(adminKeyPair));
    console.log('adminWallet: ', adminWallet.publicKey.toString());
    //devnet rpc: https://winter-flashy-dawn.solana-devnet.discover.quiknode.pro/c7023a9cfda5932a4e18ec7f381e98cc2226c22e/
    const program: Program<NftStaking> = await loadProgram(adminWallet, env, programId.toString(), rpc);
    await addCollection(program, adminWallet);
});

programCommand('addRewards')
.action(async (options) => {
    debug('options: ', options);
    const keypairPath = options.keypair;
    const env = options.env;
    const rpc = options.rpc;
    debug('keypairPath: ', keypairPath);
    const adminWallet = Keypair.fromSecretKey(new Uint8Array(adminKeyPair));
    console.log('adminWallet: ', adminWallet.publicKey.toString());
    //devnet rpc: https://winter-flashy-dawn.solana-devnet.discover.quiknode.pro/c7023a9cfda5932a4e18ec7f381e98cc2226c22e/
    const program: Program<NftStaking> = await loadProgram(adminWallet, env, programId.toString(), rpc);
    await addReward(program, adminWallet, 1000, 10);
});

programCommand('stakeNft')
    .action(async (options) => {
        debug('options: ', options);
        const keypairPath = options.keypair;
        const env = options.env;
        const rpc = options.rpc;
        debug('keypairPath: ', keypairPath);
        const adminWallet = Keypair.fromSecretKey(new Uint8Array(adminKeyPair));
        console.log('adminWallet: ', adminWallet.publicKey.toString());
        const user_wallet = Keypair.fromSecretKey(new Uint8Array(userKeyPair))
        console.log('user_wallet: ', user_wallet.publicKey.toString());
        //devnet rpc: https://winter-flashy-dawn.solana-devnet.discover.quiknode.pro/c7023a9cfda5932a4e18ec7f381e98cc2226c22e/
        const program = await loadProgram(adminWallet, env, programId.toString(), rpc);
        await stakeNft( program, user_wallet, adminWallet, mint );
    });

programCommand('unstakeNft')
    .action(async (options) => {
        debug('options: ', options);
        const keypairPath = options.keypair;
        const env = options.env;
        const rpc = options.rpc;
        debug('keypairPath: ', keypairPath);
        const adminWallet = Keypair.fromSecretKey(new Uint8Array(adminKeyPair));
        console.log('adminWallet: ', adminWallet.publicKey.toString());
        const user_wallet = Keypair.fromSecretKey(new Uint8Array(userKeyPair))
        console.log('user_wallet: ', user_wallet.publicKey.toString());
        //devnet rpc: https://winter-flashy-dawn.solana-devnet.discover.quiknode.pro/c7023a9cfda5932a4e18ec7f381e98cc2226c22e/
        const program = await loadProgram(adminWallet, env, programId.toString(), rpc);
        await unstakeNft(program, user_wallet, adminWallet, mint);
    });

programCommand('claimRewards')
    .action(async (options) => {
        debug('options: ', options);
        const keypairPath = options.keypair;
        const env = options.env;
        const rpc = options.rpc;
        debug('keypairPath: ', keypairPath);
        const adminWallet = Keypair.fromSecretKey(new Uint8Array(adminKeyPair));
        console.log('adminWallet: ', adminWallet.publicKey.toString());
        const user_wallet = Keypair.fromSecretKey(new Uint8Array(userKeyPair))
        console.log('user_wallet: ', user_wallet.publicKey.toString());
        //devnet rpc: https://winter-flashy-dawn.solana-devnet.discover.quiknode.pro/c7023a9cfda5932a4e18ec7f381e98cc2226c22e/
        const program = await loadProgram(adminWallet, env, programId.toString(), rpc);
        await claimRewards(program, user_wallet, adminWallet, mint);
    });

programCommand('withdraw')
    .action(async (options) => {
        debug('options: ', options);
        const keypairPath = options.keypair;
        const env = options.env;
        const rpc = options.rpc;
        debug('keypairPath: ', keypairPath);
        const adminWallet = Keypair.fromSecretKey(new Uint8Array(adminKeyPair));
        console.log('adminWallet: ', adminWallet.publicKey.toString());
        //devnet rpc: https://winter-flashy-dawn.solana-devnet.discover.quiknode.pro/c7023a9cfda5932a4e18ec7f381e98cc2226c22e/
        const program = await loadProgram(adminWallet, env, programId.toString(), rpc);
        // await withdrawTokens({program, operatorKeypair: adminWallet, amount: 1000});
    });

// programCommand('updateJobStatus')
//     .action(async (options) => {
//         debug('options: ', options);
//         const keypairPath = options.keypair;
//         const env = options.env;
//         const rpc = options.rpc;
//         debug('keypairPath: ', keypairPath);
//         const adminWallet = Keypair.fromSecretKey(new Uint8Array(adminKeyPair));
//         console.log('adminWallet: ', adminWallet.publicKey.toString());
//         //devnet rpc: https://winter-flashy-dawn.solana-devnet.discover.quiknode.pro/c7023a9cfda5932a4e18ec7f381e98cc2226c22e/
//         const program = await loadProgram(adminWallet, env, programId.toString(), rpc);
//         await updateJobStatus(program, adminWallet, adminWallet2);
//     });

// programCommand('initToken')
//     .action(async (options) => {
//             debug('options: ', options);
//             const keypairPath = options.keypair;
//             const env = options.env;
//             const rpc = options.rpc;
//             debug('keypairPath: ', keypairPath);
//             const adminWallet = Keypair.fromSecretKey(new Uint8Array(adminKeyPair));
//             console.log('adminWallet: ', adminWallet.publicKey.toString());
//             //devnet rpc: https://winter-flashy-dawn.solana-devnet.discover.quiknode.pro/c7023a9cfda5932a4e18ec7f381e98cc2226c22e/
//             const program = await loadProgram(adminWallet, env, programId.toString(), rpc);
//             await initToken(program, adminWallet);
//     });

// programCommand('mintToken')
//     .action(async (options) => {
//         debug('options: ', options);
//         const keypairPath = options.keypair;
//         const env = options.env;
//         const rpc = options.rpc;
//         debug('keypairPath: ', keypairPath);
//         const adminWallet = Keypair.fromSecretKey(new Uint8Array(adminKeyPair));
//         console.log('adminWallet: ', adminWallet.publicKey.toString());
//         //devnet rpc: https://winter-flashy-dawn.solana-devnet.discover.quiknode.pro/c7023a9cfda5932a4e18ec7f381e98cc2226c22e/
//         const program = await loadProgram(adminWallet, env, programId.toString(), rpc);
//         const userWallet = Keypair.fromSecretKey(new Uint8Array(userKeyPair));
//         await mintToken(program, adminWallet, userWallet.publicKey);
//     });

// programCommand('deposit')
//     .action(async (options) => {
//         debug('options: ', options);
//         const keypairPath = options.keypair;
//         const env = options.env;
//         const rpc = options.rpc;
//         debug('keypairPath: ', keypairPath);
//         const adminWallet = Keypair.fromSecretKey(new Uint8Array(adminKeyPair));
//         console.log('adminWallet: ', adminWallet.publicKey.toString());
//         //devnet rpc: https://winter-flashy-dawn.solana-devnet.discover.quiknode.pro/c7023a9cfda5932a4e18ec7f381e98cc2226c22e/
//         const userWallet = Keypair.fromSecretKey(new Uint8Array(userKeyPair));
//         const program = await loadProgram(userWallet, env, programId.toString(), rpc);
//         console.log('userWallet: ', userWallet.publicKey.toString());
//         await deposit(program, userWallet);
//     });

// programCommand('createTree')
//     .action(async (options) => {
//         debug('options: ', options);
//         const keypairPath = options.keypair;
//         const env = options.env;
//         const rpc = options.rpc;
//         debug('keypairPath: ', keypairPath);
//         const adminWalletKey = Keypair.fromSecretKey(new Uint8Array(adminKeyPair));
//         console.log('adminWallet: ', adminWalletKey.publicKey.toString());
//         //devnet rpc: https://winter-flashy-dawn.solana-devnet.discover.quiknode.pro/c7023a9cfda5932a4e18ec7f381e98cc2226c22e/
//         const userWallet = Keypair.fromSecretKey(new Uint8Array(userKeyPair));
//         const program = await loadProgram(adminWalletKey, env, programId.toString(), rpc);
//         await createTree(program, adminWalletKey);
//     });

// programCommand('appendNote')
//     .action(async (options) => {
//         debug('options: ', options);
//         const keypairPath = options.keypair;
//         const env = options.env;
//         const rpc = options.rpc;
//         debug('keypairPath: ', keypairPath);
//         const adminWallet = Keypair.fromSecretKey(new Uint8Array(adminKeyPair));
//         console.log('adminWallet: ', adminWallet.publicKey.toString());
//         //devnet rpc: https://winter-flashy-dawn.solana-devnet.discover.quiknode.pro/c7023a9cfda5932a4e18ec7f381e98cc2226c22e/
//         const userWallet = Keypair.fromSecretKey(new Uint8Array(userKeyPair));
//         const program = await loadProgram(userWallet, env, programId.toString(), rpc);
//         await appendTree(program);
//     });

// programCommand('updateNote')
//     .action(async (options) => {
//         debug('options: ', options);
//         const keypairPath = options.keypair;
//         const env = options.env;
//         const rpc = options.rpc;
//         debug('keypairPath: ', keypairPath);
//         const adminWallet = Keypair.fromSecretKey(new Uint8Array(adminKeyPair));
//         console.log('adminWallet: ', adminWallet.publicKey.toString());
//         //devnet rpc: https://winter-flashy-dawn.solana-devnet.discover.quiknode.pro/c7023a9cfda5932a4e18ec7f381e98cc2226c22e/
//         const userWallet = Keypair.fromSecretKey(new Uint8Array(userKeyPair));
//         const program = await loadProgram(userWallet, env, programId.toString(), rpc);
//         await updateNote(program, adminWallet);
//     });

// programCommand('deleteNote')
//     .action(async (options) => {
//         debug('options: ', options);
//         const keypairPath = options.keypair;
//         const env = options.env;
//         const rpc = options.rpc;
//         debug('keypairPath: ', keypairPath);
//         const adminWallet = Keypair.fromSecretKey(new Uint8Array(adminKeyPair));
//         console.log('adminWallet: ', adminWallet.publicKey.toString());
//         //devnet rpc: https://winter-flashy-dawn.solana-devnet.discover.quiknode.pro/c7023a9cfda5932a4e18ec7f381e98cc2226c22e/
//         const userWallet = Keypair.fromSecretKey(new Uint8Array(userKeyPair));
//         const program = await loadProgram(userWallet, env, programId.toString(), rpc);
//         await deleteNote(program, adminWallet);
//     });
program.parse(process.argv);