
const fs = require('fs');
const { FiveSDK } = require('../../five-sdk/dist/index.js');

async function main() {
    const fileContent = fs.readFileSync('build/five-token-template.five');
    const bytecode = fileContent;
    const abi = {
        functions: [
            {
                name: "revoke",
                index: 9,
                parameters: []
            }
        ]
    };

    // Create VM state mock
    const vmState = {
        authority: "11111111111111111111111111111111", // Mock authority
        deploy_fee_bps: 0,
        execute_fee_bps: 0
    };

    // Revoke has function index 9. Params: source, owner. (Accounts).
    // Local execution mocks accounts.
    const scriptAccount = "ScriptAccount11111111111111111111111111111";
    const user = "User11111111111111111111111111111111111111";

    const accounts = [
        { pubKey: scriptAccount, isSigner: false, isWritable: false }, // Script
        { pubKey: vmState.authority, isSigner: false, isWritable: false }, // VM State
        { pubKey: user, isSigner: true, isWritable: true }, // Payer
        // Extra accounts for revoke?
        // script(0), vm(1), user(2).
        // Revoke expects: source_account, owner.
        // FiveSDK passes accounts array to execute.
        // Accounts[0] = script. Accounts[1] = vm.
        // Accounts[2] = source_account. Accounts[3] = owner.
        // We need to provide these.
        { pubKey: "TokenAccount1111111111111111111111111111", isSigner: false, isWritable: true },
        { pubKey: user, isSigner: true, isWritable: false }
    ];

    console.log("Compiling/Preparing...");
    // Local execution requires bytecode

    console.log("Executing 'revoke' (index 9)...");
    try {
        const result = await FiveSDK.executeLocally(
            bytecode,
            "revoke",
            [], // No args (accounts are passed separately in verify/execute, but executeLocally uses mock fields?)
            // update: executeLocally signature: (bytecode, functionName, args, accounts?, callbacks?)
            { accounts, abi }
            // params
        );
        console.log("Success:", result);
    } catch (e) {
        console.error("Execution Failed:", e);
    }
}

main();
