// file: appZ/initProgram.ts

import { Program, web3, AnchorProvider } from "@coral-xyz/anchor";
import {
  TOKEN_PROGRAM_ID,
  ASSOCIATED_TOKEN_PROGRAM_ID,
  getAssociatedTokenAddress,
  createMint,
} from "@solana/spl-token";
import { AccountMeta, Keypair, PublicKey, SystemProgram, SYSVAR_RENT_PUBKEY } from "@solana/web3.js";
import { collection, tokenAddress } from "./Constants";
import { NftStaking } from "../target/types/nft_staking";
import { createAtaIfNeeded } from "./utils";
const MPL_TOKEN_METADATA_PROGRAM_ID = new PublicKey("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s");

/**
 * Initializes the Solana program state.
 * This should only be called ONCE per deployment.
 *
 * @param program - The Anchor program instance.
 * @param admin - The Keypair of the administrator/operator.
 */
export async function stakeNft(
  program: Program<NftStaking>,
  userWallet: Keypair,
  admin: Keypair,
  mint: PublicKey
) {
  const provider = program.provider as AnchorProvider;
  const PROGRAM_STATE_SEED = Buffer.from("pool");
  const [poolPDA, programStateBump] =
    PublicKey.findProgramAddressSync(
      [PROGRAM_STATE_SEED],
      program.programId
    );
  console.log(`Derived Program State PDA: ${poolPDA.toBase58()}`);
  console.log(`Derived Program State Bump: ${programStateBump}`)
  console.log(`program: `, program.programId.toString())
  console.log(`userWallet: ${userWallet.publicKey}`)

  // Derive all required PDAs
  const nftMintAddress = mint;
  const [stakeEntryPDA] = PublicKey.findProgramAddressSync(
    [Buffer.from("stake_entry"), userWallet.publicKey.toBuffer(), nftMintAddress.toBuffer()],
    program.programId
  );
  const [nftVaultPDA] = PublicKey.findProgramAddressSync(
    [Buffer.from("nft_vault"), userWallet.publicKey.toBuffer(), nftMintAddress.toBuffer()],
    program.programId
  );
  const [nftMetadataAccount] = PublicKey.findProgramAddressSync(
    [Buffer.from("metadata"), MPL_TOKEN_METADATA_PROGRAM_ID.toBuffer(), nftMintAddress.toBuffer()],
    MPL_TOKEN_METADATA_PROGRAM_ID
  );
  const userNftTokenAccount = await getAssociatedTokenAddress(nftMintAddress, userWallet.publicKey);
  console.log("\nSending transaction to initialize program...");
  try {
    const mintPubkeys = [mint];
    const tx = await (program.methods
      .stake() as any)
      .accounts({
        user: userWallet.publicKey,
        pool: poolPDA,
        nftMint: nftMintAddress,
        nftMetadataAccount,
        stakeEntry: stakeEntryPDA,
        userNftTokenAccount,
        nftVault: nftVaultPDA,
        systemProgram: SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([userWallet])
      .rpc();

    console.log(`✅ ${mintPubkeys.length} NFTs staked successfully!`);
    console.log(`Transaction signature: ${tx}`);

  } catch (error) {
    console.error("❌ Program initialization failed!");
    console.error(error);
    throw error;
  }
}