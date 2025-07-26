// // app/Constants.ts
import {clusterApiUrl, PublicKey} from "@solana/web3.js";
import * as anchor from '@coral-xyz/anchor';

// @ts-ignore
export const CLUSTERS: Cluster[] = [
    {
        name: 'mainnet-beta',
        url: 'https://api.metaplex.solana.com/',
    },
    {
        name: 'testnet',
        url: clusterApiUrl('testnet'),
    },
    {
        name: 'devnet',
        url: clusterApiUrl('devnet'),
    },
];

export const DEFAULT_CLUSTER = CLUSTERS[2];

export const programId = new anchor.web3.PublicKey('H6B58gF8oPpvEkFaRZmF9bZtD2E6KQdme3GdiGE9fEz');
export const SplAccountCompressionProgram = new anchor.web3.PublicKey('cmtDvXumGCrqC1Age74AVPhSRVXJMd8PJS91L8KbNCK');
export const logWrapperProgram = new anchor.web3.PublicKey('noopb9bkMVfRPU8AsbpTUg8AQkHtKwMYZiFUjNRtMmV');
export const adminKeyPair: number[] = [106,146,102,229,229,48,122,157,193,127,238,241,220,199,197,115,228,138,15,65,95,
    208,148,25,141,32, 237,251,105,214,125,148,192,180,179,13,87,88,99,188,100,209,59,196,195,145,90,131,174,87,20,192,
    105,214,105,219,169,68,140,173,151,0,119,216];
export const adminWallet = new PublicKey("DyF6BvoUZwpyuiYHcke1buAKxb1a9U2dEYuArW8ZoAdV");
export const backendWalletPubkey = new PublicKey("Dzfa76HDASqGu83DZGtY784tAPDHPZXwmzVHeEQRke1j");
export const backendWalletKeypair: number[] = [119,110,39,145,39,64,31,1,225,148,170,189,159,145,53,18,239,135,169,46,
    161,78,32,211,66,198,24,239,123,50,205,17,193,17,246,26,250,110,198,112,84,57,153,13,210,153,181,155,196,227,234,
    13,192,248,9,148,242,120,127,241,69,31,119,182];
export const tree = new PublicKey("CfwcyS1sTyE7Hg9CzkHNesPJ9NqV2nuoJBDdoGZhdkJQ");
export const firstComitTx = "5sEjgp9Pj3R1qmm1yRyDi58wcevaVgJCjGnHHq3B7abUjLU8ee5iusn2Ask62cYsSWRvRJUb7RMB5sziChDaKJDH";
export const firstComit = "first commit";
export const updateComit = "update commit";
export const PROGRAM_STATE_SEED = Buffer.from("pool");
export const collection = new anchor.web3.PublicKey("HzTGrd1QV4TPE3YXS8spUn59KGoidXzGLLYRMpP5DSeT");
export const mint = new anchor.web3.PublicKey("2sxPASGNkB1rTf6menbzcQh73oKou4critigrXzTJrnD");

export const tokenAddress = new anchor.web3.PublicKey('5CBgHcp64Ua9KbMyDW7z68CXy79jfHUF75LW3d1H2c9M');
// export const userKeyPair = [96,120,119,109,219,23,36,180,235,23,109,205,17,155,31,185,8,212,91,217,193,164,
//     132,18,127,252,241,192,87,181,187,235,21,207,117,128,141,190,249,46,185,255,119,83,152,61,155,148,226,199,66,237,
//     251,187,221,153,198,98,241,217,114,17,17,108];

export const userKeyPair = [119,110,39,145,39,64,31,1,225,148,170,189,159,145,53,18,239,135,169,46,161,78,32,211,66,198,
    24,239,123,50,205,17,193,17,246,26,250,110,198,112,84,57,153,13,210,153,181,155,196,227,234,13,192,248,9,148,242,120,
    127,241,69,31,119,182]