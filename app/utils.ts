// app/utils.ts
import {AccountInfo, clusterApiUrl, Connection, Keypair, PublicKey, sendAndConfirmTransaction, Transaction} from "@solana/web3.js";
import * as anchor from '@project-serum/anchor';
import fs from "fs";
import * as bs58 from 'bs58';
import {AnchorProvider, Program, Provider, Wallet} from "@coral-xyz/anchor";
import { deserialize } from "borsh"
import { NftStaking } from "../target/types/nft_staking";
import { createAssociatedTokenAccountInstruction } from "@solana/spl-token";
class NoteLog {
    leafNode: Uint8Array
    checker: PublicKey
    hash: string

    constructor(properties: {
        leafNode: Uint8Array
        checker: Uint8Array
        hash: string
    }) {
        this.leafNode = properties.leafNode
        this.checker = new PublicKey(properties.checker)
        this.hash = properties.hash
    }
}

// A map that describes the Note structure for Borsh deserialization
const NoteLogBorshSchema = new Map([
    [
        NoteLog,
        {
            kind: "struct",
            fields: [
                ["leafNode", [32]], // Array of 32 `u8`
                ["checker", [32]], // Pubkey
                ["hash", "string"],
            ],
        },
    ],
])
export async function loadProgram(
    walletKeyPair: Keypair,
    env: string,
    programId: string,
    customRpcUrl?: string,
): Promise<Program<NftStaking>> {
    if (customRpcUrl) console.log('USING CUSTOM URL', customRpcUrl);
    // @ts-ignore
    const solConnection = new anchor.web3.Connection(
        //@ts-ignore
        customRpcUrl || clusterApiUrl(env),
    );
    const walletWrapper = new Wallet(walletKeyPair);
    let provider = new AnchorProvider(solConnection, walletWrapper,  {})
    console.log('programId: ', programId);

    const idl = JSON.parse(
        fs.readFileSync("target/idl/nft_staking.json", "utf8")
    );
    return new Program(idl, provider as Provider);

}

export function loadWalletKey(keypair: any): Keypair {
    if (!keypair || keypair == '') {
        throw new Error('Keypair is required!');
    }

    const decodedKey = new Uint8Array(
        keypair.endsWith('.json') && !Array.isArray(keypair)
            ? JSON.parse(fs.readFileSync(keypair).toString())
            : bs58.decode(keypair),
    );

    const loaded = Keypair.fromSecretKey(decodedKey);
    return loaded;
}

// Helper function to create an ATA if it doesn't exist
export async function createAtaIfNeeded(
    connection: Connection,
    payer: Keypair,
    ataAddress: PublicKey,
    owner: PublicKey,
    mint: PublicKey
): Promise<void> {
    const accountInfo: AccountInfo<Buffer> | null = await connection.getAccountInfo(ataAddress);
    if (!accountInfo) {
        console.log(`ATA ${ataAddress.toBase58()} not found. Creating...`);
        const tx = new Transaction().add(
            createAssociatedTokenAccountInstruction(
                payer.publicKey,
                ataAddress,
                owner,
                mint
            )
        );
        await sendAndConfirmTransaction(connection, tx, [payer]);
    }
}