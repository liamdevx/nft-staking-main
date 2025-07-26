// file: appZ/initProgram.ts

import { Program, web3, AnchorProvider } from "@coral-xyz/anchor";
import {
  TOKEN_PROGRAM_ID,
  ASSOCIATED_TOKEN_PROGRAM_ID,
  getAssociatedTokenAddress,
  createMint,
} from "@solana/spl-token";
import { Keypair, PublicKey, SystemProgram, SYSVAR_RENT_PUBKEY } from "@solana/web3.js";
import { collection, tokenAddress } from "./Constants";
import { NftStaking } from "../target/types/nft_staking";

/**
 * Initializes the Solana program state.
 * This should only be called ONCE per deployment.
 *
 * @param program - The Anchor program instance.
 * @param admin - The Keypair of the administrator/operator.
 */
export async function initProgram(
    program: Program<NftStaking>,
    admin: Keypair
  ): Promise<{
  programStatePDA: PublicKey;
}> {
  const provider = program.provider as AnchorProvider;
  const PROGRAM_STATE_SEED = Buffer.from("pool");
  const [programStatePDA, programStateBump] =
    PublicKey.findProgramAddressSync(
      [PROGRAM_STATE_SEED],
      program.programId
    );
  console.log(`Derived Program State PDA: ${programStatePDA.toBase58()}`);
  console.log(`Derived Program State Bump: ${programStateBump}`)
  console.log(`program: `, program.programId.toString())
  const [rewardVaultPDA] = PublicKey.findProgramAddressSync([Buffer.from("reward_vault")], program.programId);
  console.log(`Derived rewardVaultPDA: ${rewardVaultPDA.toBase58()}`);
  // 2. Create a new SPL Token Mint
  // In a real scenario, you might use an existing token mint.
  // console.log("Creating a new token mint...");
  // const tokenMint = await createMint(
  //   provider.connection,
  //   admin,              // Payer for mint creation
  //   admin.publicKey,    // Mint Authority
  //   admin.publicKey,    // Freeze Authority
  //   6                   // Decimals
  // );
  // console.log(`New Token Mint created: ${tokenMint.toBase58()}`);

  // const tokenMint = tokenAddress;

  // 3. Derive the Program's Associated Token Account (ATA) for the new mint
  // const programTokenAccountPDA = await getAssociatedTokenAddress(
  //   tokenMint,
  //   programStatePDA, // The PDA is the authority of its own token account
  //   true             // Allow owner to be off-curve (since it's a PDA)
  // );
  // console.log(`Program's Token Account PDA: ${programTokenAccountPDA.toBase58()}`);

  console.log("\nSending transaction to initialize program...");
  
  try {
    // 4. Call the `initializeProgram` instruction from the smart contract
    // Note: The on-chain program has `initialize_program(ctx: Context<InitializeProgram>)`
    // which takes no extra arguments, so we call it with empty `()`.
    const txSignature = await (await program.methods
      .initializePool as any)()
      .accounts({
        admin: admin.publicKey,
        pool: programStatePDA,
        rewardMint: tokenAddress,
        rewardVault: rewardVaultPDA,
        systemProgram: SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([admin]) // The admin must sign to pay for account creation
      .rpc();

    console.log(`✅ Program initialized successfully!`);
    console.log(`Transaction Signature: ${txSignature}`);

    // 5. Fetch and verify the state
    // const state = await program.account.programState.fetch(programStatePDA);
    console.log("\nOn-chain Program State:");
    // console.log(`- Operator: ${state.operator.toBase58()}`);
    // console.log(`- Backend Wallet: ${state.backendWallet.toBase58()}`);
    // console.log(`- Token Mint: ${state.tokenMint.toBase58()}`);
    // console.log(`- Total Buyers: ${state.totalBuyers.toString()}`);

    return {
      programStatePDA,
      // tokenMint,
      // programTokenAccountPDA,
    };
  } catch (error) {
    console.error("❌ Program initialization failed!");
    console.error(error);
    throw error;
  }
}