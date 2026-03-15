# 5ive Connect4 Web

Standalone web app for Connect4, scaffolded from `five-templates/web-starter`.

## Local development

From `5ive-connect4/`:

```bash
npm run build
npm run web:install
cp web/.env.example web/.env.local
npm run web:dev
```

Then open `http://localhost:3000`.

## Notes

- Use `NEXT_PUBLIC_RPC_URL` and `NEXT_PUBLIC_FIVE_SCRIPT_ACCOUNT` to target your deployment.
- The UI executes all public Connect4 functions, including setup, match actions, and getters.
- If you do not set fixed accounts in env, use the `Provision Accounts` button.
