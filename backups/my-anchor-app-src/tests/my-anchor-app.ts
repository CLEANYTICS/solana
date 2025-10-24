import * as anchor from "@coral-xyz/anchor";
import {
  TOKEN_PROGRAM_ID,
  ASSOCIATED_TOKEN_PROGRAM_ID,
  createMint,
  getAssociatedTokenAddress,
} from "@solana/spl-token";

anchor.setProvider(anchor.AnchorProvider.env());

describe("my-anchor-app", () => {
  it("initialize_vault creates the PDA + ATA", async () => {
    const provider = anchor.getProvider();
    const program = anchor.workspace.MyAnchorApp as anchor.Program;

    // payer (the wallet in ANCHOR_WALLET)
    // @ts-ignore - NodeWallet has a .payer
    const payerKp: anchor.web3.Keypair = provider.wallet.payer;
    const payerPk = provider.wallet.publicKey;

    // 1) create a test mint (decimals 6 like USDC)
    const mint = await createMint(
      provider.connection,
      payerKp,          // fee payer + mint authority
      payerPk,          // mint authority
      null,             // freeze authority (none)
      6                 // decimals
    );

    // 2) derive your vault PDA (["vault", mint])
    const [vaultPda] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("vault"), mint.toBuffer()],
      program.programId
    );

    // 3) vault’s ATA (Associated Token Account) for that mint (owner = vault PDA)
    const vaultAta = await getAssociatedTokenAddress(
      mint,
      vaultPda,
      true,              // allow owner off-curve (PDAs are off-curve)
    );

    // 4) call initialize_vault with the correct accounts
    const sig = await program.methods
      .initializeVault()
      .accounts({
        payer: payerPk,
        mint,
        vault: vaultPda,
        vaultToken: vaultAta,
        systemProgram: anchor.web3.SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
      })
      .rpc();

    console.log("initialize_vault tx:", sig);
  });
});
