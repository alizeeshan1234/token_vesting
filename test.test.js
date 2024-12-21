const anchor = require('@project-serum/anchor');
const assert = require('assert');
    describe('token_vesting', () => {
        it('should initialize token vesting with correct values', async () => {
            // Create test accounts
            const [employeer, vestingAccount, tokenVaultAccount, mint] = await anchor.web3.PublicKey.createWithSeed(
                provider.wallet.publicKey,
                "token_vesting",
                anchor.web3.SystemProgram.programId,
                Buffer.from("token_vesting")
            );

            // Initialize token vesting
            await program.rpc.initTokenVesting(new anchor.BN(100), new anchor.BN(1630473600), new anchor.BN(1662009600), new anchor.BN(86400), {
                accounts: {
                    employeer,
                    vestingAccount,
                    tokenVaultAccount,
                    mint,
                    systemProgram: anchor.web3.SystemProgram.programId,
                    tokenProgram: anchor.web3.TokenProgram.programId,
                },
                signers: [employeer],
            });

            // Retrieve the initialized token vesting account
            const vesting = await program.account.vestingAccount.fetch(vestingAccount);

            // Assert the initialized values
            assert.ok(vesting.totalAmount.eq(new anchor.BN(100)), "Total amount should be 100");
            assert.ok(vesting.startTime.eq(new anchor.BN(1630473600)), "Start time should be 1630473600");
            assert.ok(vesting.endTime.eq(new anchor.BN(1662009600)), "End time should be 1662009600");
            assert.ok(vesting.cliffDuration.eq(new anchor.BN(86400)), "Cliff duration should be 86400");
        });

        it('should close vesting account and transfer remaining tokens to token vault', async () => {
            // Create test accounts
            const [employeer, tokenVaultAccount] = await anchor.web3.PublicKey.createWithSeed(
                provider.wallet.publicKey,
                "token_vault_account",
                anchor.web3.SystemProgram.programId,
                Buffer.from("token_vault_account")
            );

            // Close vesting account
            await program.rpc.closeVestingAccount({
                accounts: {
                    employeer,
                    tokenVaultAccount,
                    tokenProgram: anchor.web3.TokenProgram.programId,
                    systemProgram: anchor.web3.SystemProgram.programId,
                },
                signers: [employeer],
            });

            // Retrieve the closed vesting account
            const vesting = await program.account.vestingAccount.fetch(tokenVaultAccount);

            // Assert that the vesting account is closed
            assert.ok(vesting.isClosed, "Vesting account should be closed");
        });

        it('should initialize employee account with correct values', async () => {
            // Create test accounts
            const [employee, employeeTokenAccount, mint] = await anchor.web3.PublicKey.createWithSeed(
                provider.wallet.publicKey,
                "employee_token_account",
                anchor.web3.SystemProgram.programId,
                Buffer.from("employee_token_account")
            );

            // Initialize employee account
            await program.rpc.initializeEmployeeAccount({
                accounts: {
                    employee,
                    employeeTokenAccount,
                    mint,
                    systemProgram: anchor.web3.SystemProgram.programId,
                    tokenProgram: anchor.web3.TokenProgram.programId,
                },
                signers: [employee],
            });

            // Retrieve the initialized employee account
            const employeeAccount = await program.account.employeeAccount.fetch(employeeTokenAccount);

            // Assert the initialized values
            assert.ok(employeeAccount.balance.eq(new anchor.BN(0)), "Balance should be 0");
            assert.ok(employeeAccount.mint.equals(mint), "Mint should match");
        });

        it('should claim tokens and update employee account balance', async () => {
            // Create test accounts
            const [employeer, vestingAccount, tokenVaultAccount, employeeTokenAccount, mint] = await anchor.web3.PublicKey.createWithSeed(
                provider.wallet.publicKey,
                "token_vesting",
                anchor.web3.SystemProgram.programId,
                Buffer.from("token_vesting")
            );

            // Claim tokens
            await program.rpc.claimTokens({
                accounts: {
                    employeer,
                    vestingAccount,
                    tokenVaultAccount,
                    employeeTokenAccount,
                    mint,
                    tokenProgram: anchor.web3.TokenProgram.programId,
                },
                signers: [employeer],
            });

            // Retrieve the updated employee account
            const employeeAccount = await program.account.employeeAccount.fetch(employeeTokenAccount);

            // Assert that the employee account balance is updated
            assert.ok(employeeAccount.balance.gt(new anchor.BN(0)), "Balance should be greater than 0");
        });
    });
    