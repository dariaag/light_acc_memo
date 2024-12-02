import * as anchor from '@project-serum/anchor'
import {Connection, Keypair, PublicKey, SystemProgram, Transaction, TransactionInstruction} from '@solana/web3.js'
import {expect} from 'chai'
import * as dotenv from 'dotenv'
import * as fs from 'fs'
import {start} from 'solana-bankrun'

dotenv.config()

describe('my_program', () => {
    const keypairPath = process.env.ANCHOR_WALLET || './id.json'

    const connection = new Connection(process.env.ANCHOR_PROVIDER_URL || 'http://127.0.0.1:8899')
    const payer = Keypair.fromSecretKey(Uint8Array.from(JSON.parse(fs.readFileSync(keypairPath, 'utf-8'))))

    // Replace with your program ID
    const programId = new PublicKey('Cj53TGGvopGVvdnB6vT5TK4LsoU4x7XY1w1ZLbpc9gfe')

    it('Should call a custom instruction', async () => {
        // Define accounts for the transaction
        const account1 = Keypair.generate()
        const account2 = Keypair.generate()
        const account3 = Keypair.generate()
        const accounts = [
            {pubkey: account1.publicKey, isSigner: true, isWritable: false},
            {pubkey: account2.publicKey, isSigner: true, isWritable: false},
            {pubkey: account3.publicKey, isSigner: true, isWritable: false},
        ]

        const discriminator = Buffer.from(anchor.utils.bytes.utf8.encode('create_compressed_account_with_memo')).slice(
            0,
            8
        )
        const signers = [payer, account1, account2, account3] // Include all the signers for the transaction

        const memo = Buffer.from('Hello:)')
        /* discriminator: &[u8; 8],
        accounts: &[SolanaAccountInfo],
        address_seed: &[u8; 32],
        program_id: &Pubkey,
        address_merkle_context: &AddressMerkleContext,
        address_merkle_tree_root_index: u16,
        input: &[u8], */

        //const data = Buffer.concat([discriminator, memo])

        const addressSeed = new Uint8Array(32)
        addressSeed.fill(1)
        const addressMerkleContext = {
            address_merkle_tree_pubkey: new PublicKey(0),
            address_queue_pubkey: new PublicKey(1),
        }
        const addressMerkleTreeRootIndex = 0

        const data = Buffer.concat([
            discriminator,
            addressSeed,
            Buffer.from(addressMerkleContext.address_merkle_tree_pubkey.toBuffer()),
            Buffer.from(addressMerkleContext.address_queue_pubkey.toBuffer()),
            Buffer.alloc(2, addressMerkleTreeRootIndex),
            memo,
        ])

        const instruction = new TransactionInstruction({
            keys: accounts,
            programId,
            data,
        })

        const transaction = new Transaction().add(instruction)

        const txSignature = await connection.sendTransaction(transaction, signers)
        console.log('Transaction signature:', txSignature)

        await connection.confirmTransaction(txSignature, 'confirmed')

        const txDetails = await connection.getTransaction(txSignature, {commitment: 'confirmed'})
        console.log('Transaction details:', JSON.stringify(txDetails, null, 2))

        if (txDetails?.meta?.logMessages) {
            console.log('Logs:')
            txDetails.meta.logMessages.forEach((log, index) => {
                console.log(`Log #${index + 1}: ${log}`)
            })
        } else {
            console.log('No logs found for this transaction.')
        }
        const accountInfo = await connection.getAccountInfo(account1.publicKey)

        // Verifying the memo data
        const data_in_account = accountInfo?.data
        const input_data_serialized = memo
        expect(data_in_account).to.not.be.null
        console.log('DATA', data_in_account)
    })
})
