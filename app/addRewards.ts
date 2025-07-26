// file: appZ/initProgram.ts

import { Program, web3, AnchorProvider, BN } from "@coral-xyz/anchor";
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
export async function addReward(
  program: Program<NftStaking>,
  admin: Keypair,
  amount: number,
  epochs: number
) {
  const totalRewardAmount = new BN(amount);
  const numEpochs = new BN(epochs);
  const provider = program.provider as AnchorProvider;
  const PROGRAM_STATE_SEED = Buffer.from("pool");
  const [poolPDA, programStateBump] =
    PublicKey.findProgramAddressSync(
      [PROGRAM_STATE_SEED],
      program.programId
    );
  const poolState = await program.account.pool.fetch(poolPDA);
  console.log(`Derived Program State PDA: ${poolPDA.toBase58()}`);
  console.log(`poolState: ${poolState}`)
  console.log(`program: `, program.programId.toString())
  console.log("\nSending transaction to initialize program...");

  const [rewardVaultPDA] = PublicKey.findProgramAddressSync([Buffer.from("reward_vault")], program.programId);
  console.log(`rewardVaultPDA: ${rewardVaultPDA}`)
  const adminRewardTokenAccount = await getAssociatedTokenAddress(poolState.rewardMint, admin.publicKey);

  try {
    const tx = await (program.methods
      .addReward(totalRewardAmount, numEpochs) as any)
      .accounts({
        pool: poolPDA,
        admin: admin.publicKey,
        rewardVault: rewardVaultPDA,
        rewardMint: poolState.rewardMint,
        adminRewardTokenAccount: adminRewardTokenAccount,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([admin])
      .rpc();

    console.log(`Transaction signature: ${tx}`);

  } catch (error) {
    console.error("❌ Program initialization failed!");
    console.error(error);
    throw error;
  }
}