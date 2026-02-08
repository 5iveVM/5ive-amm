# Five Scripts

Helper scripts for building and deploying the Five ecosystem.

## Scripts
- `build-workspace.sh`: build all workspace components.
- `build-five-solana.sh`: build the Solana program (release or debug).
- `build-production-vm.sh`: production-oriented VM build.
- `deploy-and-init.sh`: deploy the Solana program and initialize VM state.

## Examples
```bash
./five-scripts/build-workspace.sh
./five-scripts/build-five-solana.sh
./five-scripts/build-five-solana.sh debug
./five-scripts/build-production-vm.sh
./five-scripts/deploy-and-init.sh
```

