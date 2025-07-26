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
export async function claimRewards(
  program: Program<NftStaking>,
  userWallet: Keypair,
  admin: Keypair,
  mint: PublicKey
) {
  const nftMintAddress = mint;
  const provider = program.provider as AnchorProvider;
  const PROGRAM_STATE_SEED = Buffer.from("pool"); // "program_state" bytes: [112, 114, 111, 103, 114, 97, 109, 95, 115, 116, 97, 116, 101]
  const [poolPDA, programStateBump] =
    PublicKey.findProgramAddressSync(
      [PROGRAM_STATE_SEED],
      program.programId
    );
  console.log(`Derived Program State PDA: ${poolPDA.toBase58()}`);
  console.log(`Derived Program State Bump: ${programStateBump}`)
  console.log(`program: `, program.programId.toString())
  console.log(`userWallet: ${userWallet.publicKey}`)

  const poolState = await program.account.pool.fetch(poolPDA);
  const [rewardVaultPDA] = PublicKey.findProgramAddressSync([Buffer.from("reward_vault")], program.programId);
  const [stakeEntryPDA] = PublicKey.findProgramAddressSync(
    [Buffer.from("stake_entry"), userWallet.publicKey.toBuffer(), nftMintAddress.toBuffer()],
    program.programId
  );
  const userRewardTokenAccount = await getAssociatedTokenAddress(poolState.rewardMint, userWallet.publicKey);

  console.log("\nSending transaction to initialize program...");
  try {
    const mintPubkeys = [mint];
    const tx = await (program.methods
      .claimRewards() as any)
      .accounts({
        user: userWallet.publicKey,
        pool: poolPDA,
        rewardVault: rewardVaultPDA,
        rewardMint: poolState.rewardMint,
        userRewardTokenAccount,
        stakeEntry: stakeEntryPDA,
        nftMint: nftMintAddress,
        systemProgram: SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
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