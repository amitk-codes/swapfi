import * as anchor from "@coral-xyz/anchor";
import { randomBytes } from "node:crypto";
import { Program, BN } from "@coral-xyz/anchor";
import { Swapfi } from "../target/types/swapfi";
import { createAccountsMintsAndTokenAccounts } from "@solana-developers/helpers";
import {
  getAssociatedTokenAddressSync,
  TOKEN_2022_PROGRAM_ID,
} from "@solana/spl-token";
import { assert } from "chai";

describe("SwapFi", () => {
  const provider = anchor.AnchorProvider.env();
  const connection = provider.connection;
  const payer = (provider.wallet as anchor.Wallet).payer;

  const program = anchor.workspace.Swapfi as Program<Swapfi>;
  const web3 = anchor.web3;
  const offerId = randomBytes(8);
  const providedTokenAmount = 1_000_000;
  const requestedTokenAmount = 1_000_000;
  const TOKEN_PROGRAM = TOKEN_2022_PROGRAM_ID;

  let offerCreator: anchor.web3.Keypair;
  let offerAcceptor: anchor.web3.Keypair;

  let providedTokenMint: anchor.web3.Keypair;
  let requestedTokenMint: anchor.web3.Keypair;

  let offerCreatorProvidedTokenAccount: anchor.web3.PublicKey;
  let offerCreatorRequestedTokenAccount: anchor.web3.PublicKey;
  let offerAcceptorProvidedTokenAccount: anchor.web3.PublicKey;
  let offerAcceptorRequestedTokenAccount: anchor.web3.PublicKey;

  before(async () => {
    const accountsMintsAndTokenAccounts =
      await createAccountsMintsAndTokenAccounts(
        [
          [providedTokenAmount, 0],
          [0, requestedTokenAmount],
        ],
        1 * web3.LAMPORTS_PER_SOL,
        connection,
        payer
      );

    const users = accountsMintsAndTokenAccounts.users;
    offerCreator = users[0];
    offerAcceptor = users[1];

    const mints = accountsMintsAndTokenAccounts.mints;
    providedTokenMint = mints[0];
    requestedTokenMint = mints[1];

    const tokenAccounts = accountsMintsAndTokenAccounts.tokenAccounts;
    offerCreatorProvidedTokenAccount = tokenAccounts[0][0];
    offerCreatorRequestedTokenAccount = tokenAccounts[0][1];

    offerAcceptorRequestedTokenAccount = tokenAccounts[1][0];
    offerAcceptorProvidedTokenAccount = tokenAccounts[1][1];
  });

  it("should send the tokens to vault account and create the offer account", async () => {
    const tx = await program.methods
      .createSwapOffer(
        new BN(offerId),
        new BN(providedTokenAmount),
        new BN(requestedTokenAmount)
      )
      .accounts({
        offerCreator: offerCreator.publicKey,
        providedTokenMint: providedTokenMint.publicKey,
        requestedTokenMint: requestedTokenMint.publicKey,
        tokenProgram: TOKEN_PROGRAM,
      })
      .signers([offerCreator])
      .rpc();

    console.log({ createOfferTx: tx });

    const [offerAccountPDA] = web3.PublicKey.findProgramAddressSync(
      [
        Buffer.from("offer"),
        offerCreator.publicKey.toBuffer(),
        new BN(offerId).toArrayLike(Buffer, "le", 8),
      ],
      program.programId
    );

    const vaultPDA = getAssociatedTokenAddressSync(
      providedTokenMint.publicKey,
      offerAccountPDA,
      true,
      TOKEN_PROGRAM
    );

    const vaultBalance = await connection.getTokenAccountBalance(vaultPDA);
    const formattedVaultBalance = vaultBalance.value.amount;

    assert.equal(formattedVaultBalance, providedTokenAmount.toString());

    const fetchOfferAccount = await program.account.offer.fetch(
      offerAccountPDA
    );
    console.dir({ fetchOfferAccount }, { depth: Infinity });

    assert.equal(
      fetchOfferAccount.creator.toString(),
      offerCreator.publicKey.toString()
    );
    assert.equal(
      fetchOfferAccount.providedTokenMint.toString(),
      providedTokenMint.publicKey.toString()
    );
    assert.equal(
      fetchOfferAccount.requestedTokenMint.toString(),
      requestedTokenMint.publicKey.toString()
    );
    assert.equal(
      fetchOfferAccount.requestedAmount.toNumber(),
      requestedTokenAmount
    );
  });
});
