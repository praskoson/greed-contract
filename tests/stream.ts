import * as anchor from "@project-serum/anchor";
import { Keypair, PublicKey } from "@solana/web3.js";
import { Program, Spl } from "@project-serum/anchor";
import { Stream } from "../target/types/stream";

const chai = require("chai");
chai.use(require("chai-as-promised"));
const expect = chai.expect;

describe("stream", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.Stream as Program<Stream>;
  let streamAuthority: PublicKey;

  const createMint = async () => {
    const tokenProgram = Spl.token();
    const newMint = Keypair.generate();
    const createMintAccountIx = await tokenProgram.account.mint.createInstruction(newMint);
    await tokenProgram.methods
      .initializeMint(6, program.provider.publicKey, program.provider.publicKey)
      .preInstructions([createMintAccountIx])
      .accounts({ mint: newMint.publicKey })
      .signers([newMint])
      .rpc();

    return newMint.publicKey;
  };

  it("creates a mint and gives authority", async () => {
    const mint = await createMint();
    streamAuthority = PublicKey.findProgramAddressSync(
      [Buffer.from("idk"), mint.toBuffer()],
      program.programId
    )[0];

    const tx = await program.methods.giveAuthority().accounts({ mint, streamAuthority }).rpc();

    const streamAuthorityAccount = await program.account.streamAuthority.fetch(streamAuthority);
    console.log("tx success", tx, streamAuthorityAccount);
  });

  it("creates a mint, gives authority and reclaims it", async () => {
    const mint = await createMint();
    streamAuthority = PublicKey.findProgramAddressSync(
      [Buffer.from("idk"), mint.toBuffer()],
      program.programId
    )[0];

    await program.methods.giveAuthority().accounts({ mint, streamAuthority }).rpc();
    await program.methods.reclaimAuthority().accounts({ mint, streamAuthority }).rpc();

    console.log("tx success");
  });

  it("mint a frozen token for the current payer", async () => {
    const mint = await createMint();
    streamAuthority = PublicKey.findProgramAddressSync(
      [Buffer.from("idk"), mint.toBuffer()],
      program.programId
    )[0];

    await program.methods.giveAuthority().accounts({ mint, streamAuthority }).rpc();
    const token = associatedTokenAddress(mint, program.provider.publicKey);
    const tx = await program.methods
      .mintToSelf()
      .accounts({
        token,
        mint,
        streamAuthority,
      })
      .rpc();

    console.log("tx success");
  });

  it("creates a mint, gives authority and tries to reclaim it with wrong wallet", async () => {
    const mint = await createMint();
    streamAuthority = PublicKey.findProgramAddressSync(
      [Buffer.from("idk"), mint.toBuffer()],
      program.programId
    )[0];

    await program.methods.giveAuthority().accounts({ mint, streamAuthority }).rpc();

    const wrongAuthority = Keypair.generate();
    await program.provider.connection.requestAirdrop(wrongAuthority.publicKey, 1_000_000_000);

    const tx = await program.methods
      .reclaimAuthority()
      .accounts({ mint, streamAuthority, user: wrongAuthority.publicKey })
      .transaction();

    await expect(program.provider.sendAndConfirm(tx, [wrongAuthority])).to.be.rejectedWith(Error);
  });

  it("gives authority, reclaims authority, gives authority again", async () => {
    const mint = await createMint();
    streamAuthority = PublicKey.findProgramAddressSync(
      [Buffer.from("idk"), mint.toBuffer()],
      program.programId
    )[0];

    await program.methods.giveAuthority().accounts({ mint, streamAuthority }).rpc();
    await program.methods.reclaimAuthority().accounts({ mint, streamAuthority }).rpc();
    // Give it again
    await program.methods.giveAuthority().accounts({ mint, streamAuthority }).rpc();

    console.log("tx success");
  });
});

export function associatedTokenAddress(mint: PublicKey, wallet: PublicKey): PublicKey {
  return anchor.utils.publicKey.findProgramAddressSync(
    [wallet.toBuffer(), TOKEN_PROGRAM_ID.toBuffer(), mint.toBuffer()],
    ASSOCIATED_TOKEN_PROGRAM_ID
  )[0];
}

export const TOKEN_PROGRAM_ID = new PublicKey("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA");
export const ASSOCIATED_TOKEN_PROGRAM_ID = new PublicKey(
  "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL"
);
