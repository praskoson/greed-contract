// Migrations are an early feature. Currently, they're nothing more than this
// single deploy script that's invoked from the CLI, injecting a provider
// configured from the workspace's Anchor.toml.

import { Program, workspace, setProvider } from "@project-serum/anchor";
import { Stream } from "../target/types/stream";
import { PublicKey } from "@solana/web3.js";

module.exports = async function (provider) {
  // Configure client to use the provider.
  setProvider(provider);

  // Add your deploy script here.

  const program = workspace.Stream as Program<Stream>;
  console.log(program.programId.toBase58());

  const mint = new PublicKey("EbXv52sFutWWFcRUfRxa5MHYKaBsmWxBmxKAw1ko4XEo");
  const streamAuthority = PublicKey.findProgramAddressSync(
    [Buffer.from("idk"), mint.toBuffer()],
    program.programId
  )[0];

  const tx = await program.methods.giveAuthority().accounts({ mint, streamAuthority }).rpc();
  const streamAuthorityAccount = await program.account.streamAuthority.fetch(streamAuthority);
  console.log("tx success", tx, streamAuthorityAccount);
};
