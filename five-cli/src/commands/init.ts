// Init command.

import { writeFile, mkdir, access, readFile } from 'fs/promises';
import { join, resolve, dirname, basename } from 'path';
import { fileURLToPath } from 'url';
import chalk from 'chalk';
import ora from 'ora';
import { success as uiSuccess, warning as uiWarning, uiColors } from '../utils/cli-ui.js';

import {
  CommandDefinition,
  CommandContext,
  ProjectConfig,
  CompilationTarget
} from '../types.js';

export const initCommand: CommandDefinition = {
  name: 'init',
  description: 'Initialize new project',
  aliases: ['new', 'create'],
  
  options: [
    {
      flags: '-t, --template <template>',
      description: 'Project template',
      choices: ['basic', 'defi', 'nft', 'game', 'dao'],
      defaultValue: 'basic'
    },
    {
      flags: '--target <target>',
      description: 'Default compilation target',
      choices: ['vm', 'solana', 'debug', 'test'],
      defaultValue: 'vm'
    },
    {
      flags: '--name <name>',
      description: 'Project name (default: directory name)',
      required: false
    },
    {
      flags: '--description <desc>',
      description: 'Project description',
      required: false
    },
    {
      flags: '--author <author>',
      description: 'Project author',
      required: false
    },
    {
      flags: '--license <license>',
      description: 'Project license',
      defaultValue: 'MIT'
    },
    {
      flags: '--no-git',
      description: 'Skip git repository initialization',
      defaultValue: false
    },
    {
      flags: '--no-examples',
      description: 'Skip example files',
      defaultValue: false
    }
  ],

  arguments: [
    {
      name: 'directory',
      description: 'Project directory (default: current directory)',
      required: false
    }
  ],

  examples: [
    {
      command: '5ive init',
      description: 'Initialize project in current directory'
    },
    {
      command: '5ive init my-project',
      description: 'Create new project in my-project directory'
    },
    {
      command: '5ive init my-defi --template defi --target solana',
      description: 'Create DeFi project targeting Solana'
    },
    {
      command: '5ive init game --template game --no-git',
      description: 'Create game project without git initialization'
    }
  ],

  handler: async (args: string[], options: any, context: CommandContext): Promise<void> => {
    const { logger } = context;
    
    try {
      // Determine project directory
      const projectDir = args[0] || process.cwd();
      const inferredName = basename(resolve(projectDir)) || 'five-project';
      const projectName = options.name || inferredName;
      
      logger.info(`Initializing 5IVE VM project: ${projectName}`);
      
      // Check if directory exists and is empty
      await checkProjectDirectory(projectDir, logger);
      
      // Create project structure
      const spinner = ora('Creating project structure...').start();
      await createProjectStructure(projectDir, options.template);
      spinner.succeed('Project structure created');
      
      // Generate project configuration
      spinner.start('Generating configuration files...');
      await generateProjectConfig(projectDir, projectName, options);
      await generatePackageJson(projectDir, projectName, options);
      await generateClientScaffold(projectDir, templateToClientFunctions(options.template));
      spinner.succeed('Configuration files generated');

      // Generate stdlib usage docs (compiler ships bundled stdlib in v1)
      spinner.start('Generating standard library docs...');
      await generateStdlibScaffold(projectDir, options.template);
      spinner.succeed('Standard library docs generated');

      // Generate agent playbooks (always, even with --no-examples)
      spinner.start('Generating AGENTS playbooks...');
      await generateAgentPlaybooks(projectDir);
      spinner.succeed('AGENTS playbooks generated');
      
      // Generate source files
      if (!options.noExamples) {
        spinner.start('Generating example files...');
        await generateExampleFiles(projectDir, options.template);
        spinner.succeed('Example files generated');
      }
      
      // Initialize git repository
      if (!options.noGit) {
        spinner.start('Initializing git repository...');
        await initializeGitRepository(projectDir);
        spinner.succeed('Git repository initialized');
      }
      
      // Display success message
      displaySuccessMessage(projectDir, projectName, options);
      
    } catch (error) {
      logger.error('Project initialization failed:', error);
      throw error;
    }
  }
};

async function checkProjectDirectory(projectDir: string, logger: any): Promise<void> {
  try {
    await access(projectDir);
    
    // Directory exists, check if it's empty
    const { readdir } = await import('fs/promises');
    const files = await readdir(projectDir);
    
    if (files.length > 0) {
      logger.warn(`Directory ${projectDir} is not empty`);
      
      // Check for existing 5IVE project
      const hasConfig = files.includes('five.toml') || files.includes('package.json');
      if (hasConfig) {
        throw new Error('Directory already contains a project configuration');
      }
    }
  } catch (error: any) {
    if (error.code === 'ENOENT') {
      // Directory doesn't exist, create it
      await mkdir(projectDir, { recursive: true });
    } else {
      throw error;
    }
  }
}

async function createProjectStructure(projectDir: string, template: string): Promise<void> {
  const dirs = [
    'src',
    'tests',
    'client',
    'client/src',
    'client/scripts',
    'examples',
    'build',
    'docs',
    '.five'
  ];
  
  // Add template-specific directories
  switch (template) {
    case 'defi':
      dirs.push('src/protocols', 'src/tokens', 'src/pools');
      break;
    case 'nft':
      dirs.push('src/collections', 'src/metadata', 'assets');
      break;
    case 'game':
      dirs.push('src/entities', 'src/systems', 'src/components', 'assets');
      break;
    case 'dao':
      dirs.push('src/governance', 'src/treasury', 'src/proposals');
      break;
  }
  
  for (const dir of dirs) {
    await mkdir(join(projectDir, dir), { recursive: true });
  }
}

async function generateProjectConfig(
  projectDir: string, 
  projectName: string, 
  options: any
): Promise<void> {
  const config: ProjectConfig = {
    name: projectName,
    version: '0.1.0',
    description: options.description || `A 5IVE VM project`,
    sourceDir: 'src',
    buildDir: 'build',
    entryPoint: 'src/main.v',
    target: options.target as CompilationTarget,
    optimizations: {
      enableCompression: true,
      enableConstraintOptimization: true,
      optimizationLevel: 'production'
    },
    dependencies: []
  };
  
  const configContent = generateTomlConfig(config);
  await writeFile(join(projectDir, 'five.toml'), configContent);
}

async function generatePackageJson(
  projectDir: string,
  projectName: string,
  options: any
): Promise<void> {
  const packageJson = {
    name: projectName.toLowerCase().replace(/[^a-z0-9-]/g, '-'),
    version: '0.1.0',
    description: options.description || 'A 5IVE VM project',
    author: options.author || '',
    license: options.license,
    scripts: {
      build: '5ive build',
      test: '5ive test',
      deploy: '5ive deploy',
      'build:release': '5ive build -O 3',
      'build:debug': '5ive build --debug',
      'watch': '5ive build --watch',
      'client:build': 'npm --prefix client install && npm --prefix client run build',
      'client:run': 'npm --prefix client install && npm --prefix client run run'
    },
    devDependencies: {
      '@5ive-tech/cli': '^1.0.0'
    },
    keywords: [
      'five-vm',
      'blockchain',
      'solana',
      'smart-contracts'
    ]
  };
  
  await writeFile(
    join(projectDir, 'package.json'), 
    JSON.stringify(packageJson, null, 2)
  );
}

async function generateClientScaffold(
  projectDir: string,
  preferredFunctions: string[]
): Promise<void> {
  await writeFile(
    join(projectDir, 'client/package.json'),
    JSON.stringify(
      {
        name: `${projectDir.split('/').pop() || 'five-project'}-client`,
        private: true,
        version: '0.1.0',
        type: 'module',
        scripts: {
          build: 'tsc -p tsconfig.json',
          check: 'tsc --noEmit -p tsconfig.json',
          run: 'npm run build && node dist/main.js'
        },
        dependencies: {
          '@5ive-tech/sdk': '^1.1.7',
          '@solana/web3.js': '^1.98.4'
        },
        devDependencies: {
          '@types/node': '^20.0.0',
          typescript: '^5.9.2'
        }
      },
      null,
      2
    ) + '\n'
  );

  await writeFile(
    join(projectDir, 'client/tsconfig.json'),
    JSON.stringify(
      {
        compilerOptions: {
          target: 'ES2022',
          module: 'NodeNext',
          moduleResolution: 'NodeNext',
          strict: true,
          esModuleInterop: true,
          skipLibCheck: true,
          outDir: 'dist'
        },
        include: ['main.ts']
      },
      null,
      2
    ) + '\n'
  );

  await writeFile(
    join(projectDir, 'client/README.md'),
    `# Node Client Starter

This client is designed for on-chain execution on devnet/mainnet using \`FiveProgram\` + ABI from \`../build/main.five\`.

## Quickstart

\`\`\`bash
# From project root
npm run build
cd client
npm install
npm run run
\`\`\`

The starter is self-contained:
1. Uses a default devnet RPC URL in code.
2. Creates \`client/script-account.json\` on first run.
3. Uses \`~/.config/solana/id.json\` if available, otherwise creates \`client/payer.json\`.

## Notes

1. \`client/main.ts\` demonstrates instruction building for your starter contract.
2. It sends and confirms on-chain transactions, then prints signature, \`meta.err\`, and CU.
3. For account-required functions, set account mappings directly in \`ACCOUNT_OVERRIDES\` in \`client/main.ts\`.
4. Expand this file as your contract grows; keep it aligned with \`tests/main.test.v\`.
`
  );

  await writeFile(
    join(projectDir, 'client/main.ts'),
    getTemplateClientMain(preferredFunctions)
  );
}

async function generateExampleFiles(projectDir: string, template: string): Promise<void> {
  // Generate main source file
  const mainFile = getTemplateMainFile(template);
  await writeFile(join(projectDir, 'src/main.v'), mainFile);
  
  // Generate test file
  const testFile = getTemplateTestFile(template);
  await writeFile(join(projectDir, 'tests/main.test.v'), testFile);
  const fixtureFile = getTemplateOnChainFixture(template);
  await writeFile(join(projectDir, 'tests/main.test.json'), fixtureFile);
  
  // Generate README
  const readme = generateReadme(template);
  await writeFile(join(projectDir, 'README.md'), readme);
  
  // Generate .gitignore
  const gitignore = generateGitignore();
  await writeFile(join(projectDir, '.gitignore'), gitignore);
}

async function initializeGitRepository(projectDir: string): Promise<void> {
  const { execSync } = await import('child_process');
  
  try {
    execSync('git init', { cwd: projectDir, stdio: 'ignore' });
    execSync('git add .', { cwd: projectDir, stdio: 'ignore' });
    execSync('git commit -m "Initial commit"', { cwd: projectDir, stdio: 'ignore' });
  } catch (error) {
    // Git initialization is optional, don't fail the entire process
    console.warn(uiWarning('Git initialization failed'));
  }
}

function generateTomlConfig(config: ProjectConfig): string {
  return `# 5IVE VM Project Configuration
[project]
name = "${config.name}"
version = "${config.version}"
description = "${config.description}"
source_dir = "${config.sourceDir}"
build_dir = "${config.buildDir}"
entry_point = "${config.entryPoint || 'src/main.v'}"
target = "${config.target}"

[optimizations]
enable_compression = ${config.optimizations.enableCompression}
enable_constraint_optimization = ${config.optimizations.enableConstraintOptimization}
optimization_level = "${config.optimizations.optimizationLevel}"

[dependencies]
# Add project dependencies here
# example = { path = "../example" }
# future: stdlib package source (v1 uses compiler-bundled stdlib)
# five-stdlib = { version = "0.1.0" }

[build]
# Custom build settings
max_bytecode_size = 1048576  # 1MB
target_compute_units = 200000

[deploy]
# Deployment settings
network = "devnet"
# program_id = "your-program-id"
`;
}

async function generateStdlibScaffold(projectDir: string, _template: string): Promise<void> {
  const content = await loadStdlibAsset('docs/STDLIB.md');
  await writeFile(join(projectDir, 'docs/STDLIB.md'), content);
}

async function loadStdlibAsset(relativePath: string): Promise<string> {
  const __filename = fileURLToPath(import.meta.url);
  const __dirname = dirname(__filename);
  const candidates = [
    // Monorepo path when running from src or dist
    resolve(__dirname, '../../../five-stdlib', relativePath),
    // Bundled CLI asset path (works in packaged npm tarball)
    resolve(__dirname, '../../assets/stdlib', relativePath),
    resolve(__dirname, '../assets/stdlib', relativePath),
    // Fallback co-located path if packaged with CLI in the future
    resolve(__dirname, '../stdlib', relativePath),
    resolve(process.cwd(), 'five-stdlib', relativePath)
  ];

  for (const candidate of candidates) {
    try {
      return await readFile(candidate, 'utf8');
    } catch {
      // try next
    }
  }

  throw new Error(`Failed to load stdlib asset: ${relativePath}`);
}

function getStdlibPreludeBanner(): string {
  return `// 5IVE bundled stdlib (v1, compiler-provided)
// Canonical explicit imports:
// use std::builtins;
// use std::interfaces::spl_token;
// use std::interfaces::system_program;
// Call interface methods via module aliases:
// spl_token::transfer(...);
// system_program::transfer(...);

`;
}

function getTemplateMainFile(template: string): string {
  const templates: Record<string, string> = {
    basic: `${getStdlibPreludeBanner()}// Basic 5ive DSL program (valid-first starter)

account Counter {
    value: u64;
    authority: pubkey;
}

pub init_counter(
    counter: Counter @mut,
    authority: account @signer
) {
    counter.value = 0;
    counter.authority = authority.key;
}

pub increment(
    counter: Counter @mut,
    authority: account @signer
) {
    require(counter.authority == authority.key);
    counter.value = counter.value + 1;
}

pub get_value(counter: Counter) -> u64 {
    return counter.value;
}
`,

    defi: `${getStdlibPreludeBanner()}// DeFi Protocol on 5IVE VM
script DefiProtocol {
    init() {
        log("DeFi Protocol initialized");
    }
    
    constraints {
        // Ensure minimum liquidity
        require(get_balance() >= 1000, "Insufficient liquidity");
        
        // Validate price oracle
        let price = get_price_oracle();
        require(price > 0, "Invalid price");
    }
}

account LiquidityPool {
    token_a_amount: u64,
    token_b_amount: u64,
    total_shares: u64,
    fee_rate: u64
}

instruction swap(amount_in: u64, token_in: string) -> u64 {
    let pool = load_account<LiquidityPool>(0);
    
    if token_in == "A" {
        let amount_out = (amount_in * pool.token_b_amount) / (pool.token_a_amount + amount_in);
        pool.token_a_amount += amount_in;
        pool.token_b_amount -= amount_out;
        amount_out
    } else {
        let amount_out = (amount_in * pool.token_a_amount) / (pool.token_b_amount + amount_in);
        pool.token_b_amount += amount_in;
        pool.token_a_amount -= amount_out;
        amount_out
    }
}

instruction add_liquidity(amount_a: u64, amount_b: u64) -> u64 {
    let pool = load_account<LiquidityPool>(0);
    
    let shares = if pool.total_shares == 0 {
        (amount_a * amount_b).sqrt()
    } else {
        min(
            (amount_a * pool.total_shares) / pool.token_a_amount,
            (amount_b * pool.total_shares) / pool.token_b_amount
        )
    };
    
    pool.token_a_amount += amount_a;
    pool.token_b_amount += amount_b;
    pool.total_shares += shares;
    
    shares
}
`,

    nft: `${getStdlibPreludeBanner()}// NFT Collection on 5IVE VM
script NFTCollection {
    init() {
        log("NFT Collection initialized");
    }
    
    constraints {
        // Ensure valid mint authority
        require(is_mint_authority(), "Invalid mint authority");
        
        // Check collection size limits
        let current_supply = get_current_supply();
        require(current_supply < 10000, "Max supply reached");
    }
}

account NFTMetadata {
    name: string,
    symbol: string,
    uri: string,
    creator: pubkey,
    collection: pubkey,
    is_mutable: bool
}

instruction mint_nft(to: pubkey, metadata_uri: string) -> pubkey {
    let nft_id = derive_pda("nft", [to, get_clock().slot]);
    
    let metadata = NFTMetadata {
        name: "5IVE VM NFT",
        symbol: "FVM",
        uri: metadata_uri,
        creator: get_signer(),
        collection: get_program_id(),
        is_mutable: true
    };
    
    create_account(nft_id, metadata);
    log("NFT minted successfully");
    
    nft_id
}

instruction transfer_nft(nft_id: pubkey, from: pubkey, to: pubkey) {
    require(is_signer(from), "Invalid signature");
    
    let metadata = load_account<NFTMetadata>(nft_id);
    require(metadata.creator == from, "Not owner");
    
    // Update ownership (simplified)
    metadata.creator = to;
    save_account(nft_id, metadata);
    
    emit TransferEvent { from, to, nft_id };
}

event TransferEvent {
    from: pubkey,
    to: pubkey,
    nft_id: pubkey
}
`,

    game: `${getStdlibPreludeBanner()}// Game Logic on 5IVE VM
script GameEngine {
    init() {
        log("Game Engine initialized");
    }
    
    constraints {
        // Validate player actions
        let player = get_player();
        require(player.is_active, "Player not active");
        
        // Check game state
        let game_state = get_game_state();
        require(game_state == "active", "Game not active");
    }
}

account Player {
    id: pubkey,
    level: u64,
    experience: u64,
    health: u64,
    position_x: u64,
    position_y: u64,
    inventory: [u64; 10],
    is_active: bool
}

account GameWorld {
    width: u64,
    height: u64,
    players_count: u64,
    started_at: u64
}

instruction move_player(direction: string, distance: u64) {
    let player = load_account<Player>(get_signer());
    
    match direction {
        "north" => player.position_y += distance,
        "south" => player.position_y -= distance,
        "east" => player.position_x += distance,
        "west" => player.position_x -= distance,
        _ => require(false, "Invalid direction")
    }
    
    // Validate bounds
    let world = load_account<GameWorld>(0);
    require(player.position_x < world.width, "Out of bounds");
    require(player.position_y < world.height, "Out of bounds");
    
    save_account(get_signer(), player);
    emit PlayerMoved { player: get_signer(), x: player.position_x, y: player.position_y };
}

instruction level_up() {
    let player = load_account<Player>(get_signer());
    
    let required_exp = player.level * 100;
    require(player.experience >= required_exp, "Insufficient experience");
    
    player.level += 1;
    player.experience -= required_exp;
    player.health = 100; // Full heal on level up
    
    save_account(get_signer(), player);
    emit LevelUp { player: get_signer(), new_level: player.level };
}

event PlayerMoved {
    player: pubkey,
    x: u64,
    y: u64
}

event LevelUp {
    player: pubkey,
    new_level: u64
}
`,

    dao: `${getStdlibPreludeBanner()}// DAO Governance on 5IVE VM
script DAOGovernance {
    init() {
        log("DAO Governance initialized");
    }
    
    constraints {
        // Validate governance token
        let token_balance = get_token_balance();
        require(token_balance > 0, "No governance tokens");
        
        // Check proposal validity
        let proposal_id = get_current_proposal();
        if proposal_id > 0 {
            let proposal = get_proposal(proposal_id);
            require(proposal.is_active, "Proposal not active");
        }
    }
}

account Proposal {
    id: u64,
    title: string,
    description: string,
    proposer: pubkey,
    votes_for: u64,
    votes_against: u64,
    start_time: u64,
    end_time: u64,
    is_active: bool,
    is_executed: bool
}

account Vote {
    proposal_id: u64,
    voter: pubkey,
    amount: u64,
    is_for: bool
}

instruction create_proposal(title: string, description: string, duration: u64) -> u64 {
    let proposer_balance = get_token_balance();
    require(proposer_balance >= 1000, "Insufficient tokens to propose");
    
    let proposal_id = get_next_proposal_id();
    let current_time = get_clock().unix_timestamp;
    
    let proposal = Proposal {
        id: proposal_id,
        title,
        description,
        proposer: get_signer(),
        votes_for: 0,
        votes_against: 0,
        start_time: current_time,
        end_time: current_time + duration,
        is_active: true,
        is_executed: false
    };
    
    create_account(derive_pda("proposal", [proposal_id]), proposal);
    emit ProposalCreated { id: proposal_id, proposer: get_signer() };
    
    proposal_id
}

instruction vote(proposal_id: u64, amount: u64, is_for: bool) {
    let voter_balance = get_token_balance();
    require(voter_balance >= amount, "Insufficient token balance");
    
    let proposal = load_account<Proposal>(derive_pda("proposal", [proposal_id]));
    require(proposal.is_active, "Proposal not active");
    require(get_clock().unix_timestamp <= proposal.end_time, "Voting period ended");
    
    // Check if already voted
    let vote_account = derive_pda("vote", [proposal_id, get_signer()]);
    require(!account_exists(vote_account), "Already voted");
    
    // Record vote
    let vote = Vote {
        proposal_id,
        voter: get_signer(),
        amount,
        is_for
    };
    
    create_account(vote_account, vote);
    
    // Update proposal vote counts
    if is_for {
        proposal.votes_for += amount;
    } else {
        proposal.votes_against += amount;
    }
    
    save_account(derive_pda("proposal", [proposal_id]), proposal);
    emit VoteCast { proposal_id, voter: get_signer(), amount, is_for };
}

event ProposalCreated {
    id: u64,
    proposer: pubkey
}

event VoteCast {
    proposal_id: u64,
    voter: pubkey,
    amount: u64,
    is_for: bool
}
`
  };
  
  return templates[template] || templates.basic;
}

/**
 * Get template test file content
 */
function getTemplateTestFile(template: string): string {
  // Generate template-specific test functions
  const templates: Record<string, string> = {
    basic: `// Tests for ${template} template
// Use @test-params to specify function parameters for testing.
// For non-void functions the last value is the expected result.
// Space-separated and JSON-array formats are both supported.
// Keep client/main.ts calls aligned with these starter semantics.

// @test-params 10 20 30
pub test_add(a: u64, b: u64) -> u64 {
    return a + b;
}

// @test-params 5 2 10
pub test_multiply(a: u64, b: u64) -> u64 {
    return a * b;
}

// @test-params
pub test_initialization() {
    log("Initialization test passed");
}
`,

    defi: `// Tests for ${template} template
// Test DeFi protocol functionality

// @test-params 1000 2000 3000
pub test_deposit(amount: u64) -> u64 {
    // Simulate deposit logic
    let fee = (amount * 1) / 100;  // 1% fee
    return amount - fee;
}

// @test-params 100 50 50
pub test_swap_calculation(pool_a: u64, amount: u64) -> u64 {
    // Swap calculation
    let result = (amount * pool_a) / (pool_a + amount);
    return result;
}
`,

    nft: `// Tests for ${template} template
// Test NFT functionality

// @test-params
pub test_mint_nft() -> bool {
    // Test NFT minting
    log("NFT mint test passed");
    return true;
}

// @test-params
pub test_transfer_nft() -> bool {
    // Test NFT transfer
    log("NFT transfer test passed");
    return true;
}
`,

    game: `// Tests for ${template} template
// Test game logic

// @test-params 5 3 true
pub test_move_validation(x: u64, y: u64) -> bool {
    // Validate game world bounds
    let max_x = 100u64;
    let max_y = 100u64;
    return x < max_x && y < max_y;
}

// @test-params 1 100 101
pub test_level_up(level: u64, experience: u64) -> u64 {
    // Calculate experience needed for next level
    let required_exp = level * 100;
    if experience >= required_exp {
        return level + 1;
    }
    return level;
}
`,

    dao: `// Tests for ${template} template
// Test DAO governance

// @test-params 1000 50 true
pub test_vote_power(tokens: u64, vote_amount: u64) -> bool {
    // Test vote power calculation
    return vote_amount <= tokens;
}

// @test-params 1000 600 true
pub test_proposal_threshold(token_balance: u64, threshold: u64) -> bool {
    // Test if balance meets proposal threshold
    return token_balance >= threshold;
}
`
  };

  return templates[template] || templates.basic;
}

function getTemplateOnChainFixture(template: string): string {
  const fixtures: Record<string, any> = {
    basic: {
      accounts: {
        sample_account: {
          owner: 'system',
          lamports: 1000000,
          data_len: 0,
          is_writable: true
        }
      },
      tests: {
        test_add: {
          parameters: [10, 20],
          expected: { success: true }
        }
      }
    },
    defi: {
      accounts: {
        pool_account: {
          owner: 'system',
          lamports: 1000000,
          data_len: 64,
          is_writable: true
        }
      },
      tests: {
        test_deposit: {
          parameters: [1000],
          expected: { success: true }
        }
      }
    },
    nft: {
      accounts: {},
      tests: {
        test_mint_nft: {
          expected: { success: true }
        }
      }
    },
    game: {
      accounts: {},
      tests: {
        test_move_validation: {
          parameters: [5, 3],
          expected: { success: true }
        }
      }
    },
    dao: {
      accounts: {
        proposal_account: {
          owner: 'system',
          lamports: 1000000,
          data_len: 128,
          is_writable: true
        }
      },
      tests: {
        test_vote_power: {
          parameters: [1000, 50],
          expected: { success: true }
        }
      }
    }
  };

  return JSON.stringify(fixtures[template] || fixtures.basic, null, 2) + '\n';
}

function templateToClientFunctions(template: string): string[] {
  const map: Record<string, string[]> = {
    basic: ['init_counter', 'get_value'],
    defi: ['swap', 'add_liquidity'],
    nft: ['mint_nft', 'transfer_nft'],
    game: ['move_player', 'level_up'],
    dao: ['create_proposal', 'vote']
  };
  return map[template] || map.basic;
}

function getTemplateClientMain(preferredFunctions: string[]): string {
  const preferredArray = JSON.stringify(preferredFunctions);
  return `import { readFile } from 'fs/promises';
import { join } from 'path';
import { homedir } from 'os';
import {
  Connection,
  Keypair,
  PublicKey,
  Transaction,
  TransactionInstruction,
  sendAndConfirmTransaction
} from '@solana/web3.js';
import { FiveProgram, FiveSDK } from '@5ive-tech/sdk';

type AbiParameter = {
  name: string;
  is_account?: boolean;
  param_type?: string;
  type?: string;
};

const DEVNET_RPC_URL = 'https://api.devnet.solana.com';
const DEVNET_FIVE_VM_PROGRAM_ID = '4Qxf3pbCse2veUgZVMiAm3nWqJrYo2pT4suxHKMJdK1d';
const SCRIPT_ACCOUNT_FILE = join(process.cwd(), 'script-account.json');
const FALLBACK_PAYER_FILE = join(process.cwd(), 'payer.json');
const ACCOUNT_OVERRIDES: Record<string, Record<string, string>> = {
  // Example:
  // init_counter: {
  //   counter: '<COUNTER_PUBKEY>',
  //   authority: '<AUTHORITY_PUBKEY>'
  // }
};

function normalizePath(path: string): string {
  if (path.startsWith('~/')) {
    return join(homedir(), path.slice(2));
  }
  return path;
}

async function loadPayer(): Promise<Keypair> {
  const defaultPath = normalizePath('~/.config/solana/id.json');
  try {
    const secret = JSON.parse(await readFile(defaultPath, 'utf8')) as number[];
    return Keypair.fromSecretKey(new Uint8Array(secret));
  } catch {
    try {
      const secret = JSON.parse(await readFile(FALLBACK_PAYER_FILE, 'utf8')) as number[];
      return Keypair.fromSecretKey(new Uint8Array(secret));
    } catch {
      const generated = Keypair.generate();
      const { writeFile } = await import('fs/promises');
      await writeFile(FALLBACK_PAYER_FILE, JSON.stringify(Array.from(generated.secretKey), null, 2) + '\\n');
      return generated;
    }
  }
}

async function loadOrCreateScriptAccount(): Promise<string> {
  try {
    const saved = JSON.parse(await readFile(SCRIPT_ACCOUNT_FILE, 'utf8')) as { pubkey?: string };
    if (saved.pubkey) return saved.pubkey;
  } catch {
    // create below
  }
  const kp = Keypair.generate();
  const { writeFile } = await import('fs/promises');
  await writeFile(
    SCRIPT_ACCOUNT_FILE,
    JSON.stringify(
      {
        pubkey: kp.publicKey.toBase58(),
        secretKey: Array.from(kp.secretKey)
      },
      null,
      2
    ) + '\\n'
  );
  return kp.publicKey.toBase58();
}

function placeholderPubkey(): string {
  return Keypair.generate().publicKey.toBase58();
}

function getAccountOverrides(functionName: string): Record<string, string> {
  return ACCOUNT_OVERRIDES[functionName] || ACCOUNT_OVERRIDES['*'] || {};
}

function parseComputeUnitsFromLogs(logs: string[] | null | undefined): number | undefined {
  if (!logs) return undefined;
  for (const line of logs) {
    const match = line.match(/consumed\\s+(\\d+)\\s+of/i);
    if (match) return Number(match[1]);
  }
  return undefined;
}

function defaultValueForType(typeName: string | undefined): any {
  const normalized = (typeName || '').toLowerCase();
  if (normalized === 'bool' || normalized === 'boolean') return true;
  if (normalized.startsWith('string')) return 'demo';
  if (normalized === 'pubkey') return placeholderPubkey();
  return 1;
}

async function run(): Promise<void> {
  const artifactPath = join(process.cwd(), '..', 'build', 'main.five');
  const artifactText = await readFile(artifactPath, 'utf8');
  const { abi } = await FiveSDK.loadFiveFile(artifactText);

  const connection = new Connection(DEVNET_RPC_URL, 'confirmed');
  const payer = await loadPayer();
  const scriptAccount = await loadOrCreateScriptAccount();
  const program = FiveProgram.fromABI(scriptAccount, abi, {
    fiveVMProgramId: DEVNET_FIVE_VM_PROGRAM_ID
  });
  const fiveVmProgramId = program.getFiveVMProgramId();

  const preferred = ${preferredArray} as string[];
  const available = program.getFunctions();
  const targets = preferred.filter((name) => available.includes(name));
  if (targets.length === 0 && available.length > 0) {
    targets.push(available[0]);
  }

  if (targets.length === 0) {
    throw new Error('No functions found in ABI. Run npm run build first.');
  }

  console.log('[client] Loaded ABI from ../build/main.five');
  console.log('[client] RPC:', DEVNET_RPC_URL);
  console.log('[client] Payer:', payer.publicKey.toBase58());
  console.log('[client] Script account:', scriptAccount);
  console.log('[client] Five VM program id:', fiveVmProgramId);
  console.log('[client] Mode: on-chain');
  console.log('[client] Target functions:', targets.join(', '));

  for (const functionName of targets) {
    const functionDef: any = program.getFunction(functionName);
    const params: AbiParameter[] = functionDef?.parameters || [];
    const accountArgs: Record<string, string> = getAccountOverrides(functionName);
    const dataArgs: Record<string, any> = {};

    for (const param of params) {
      if (param.is_account && !accountArgs[param.name]) {
        const attributes = (param as any).attributes || [];
        if (Array.isArray(attributes) && attributes.includes('signer')) {
          accountArgs[param.name] = payer.publicKey.toBase58();
        } else {
          accountArgs[param.name] = placeholderPubkey();
        }
      } else {
        dataArgs[param.name] = defaultValueForType(param.param_type || param.type);
      }
    }

    let builder = program.function(functionName);
    if (Object.keys(accountArgs).length > 0) {
      builder = builder.accounts(accountArgs);
    }
    if (Object.keys(dataArgs).length > 0) {
      builder = builder.args(dataArgs);
    }

    const instruction = await builder.instruction();
    console.log('\\n[client] function:', functionName);
    console.log('[client] instruction bytes:', Buffer.from(instruction.data, 'base64').length);
    console.log('[client] account metas:', instruction.keys.length);

    const txIx = new TransactionInstruction({
      programId: new PublicKey(instruction.programId),
      keys: instruction.keys.map((k) => ({
        pubkey: new PublicKey(k.pubkey),
        isSigner: k.isSigner,
        isWritable: k.isWritable
      })),
      data: Buffer.from(instruction.data, 'base64')
    });
    const tx = new Transaction().add(txIx);
    const signature = await sendAndConfirmTransaction(connection, tx, [payer], { commitment: 'confirmed' });
    const txDetails = await connection.getTransaction(signature, {
      commitment: 'confirmed',
      maxSupportedTransactionVersion: 0
    });
    const metaErr = txDetails?.meta?.err ?? null;
    const computeUnits =
      txDetails?.meta?.computeUnitsConsumed ?? parseComputeUnitsFromLogs(txDetails?.meta?.logMessages);

    console.log('[client] signature:', signature);
    console.log('[client] meta.err:', metaErr);
    console.log('[client] compute units:', computeUnits ?? 'n/a');
    if (metaErr !== null) {
      throw new Error('on-chain execution failed');
    }
  }
}

run().catch((error) => {
  console.error('[client] failed:', error instanceof Error ? error.message : String(error));
  process.exit(1);
});
`;
}

/**
 * Generate README content
 */
function generateReadme(template: string): string {
  return `# 5IVE VM Project

A ${template} project built with 5IVE VM.

## Getting Started

### Prerequisites

- Node.js 18+
- 5IVE CLI: \`npm install -g @5ive-tech/cli\`

### Building

\`\`\`bash
# Compile the project
npm run build

# Compile with optimizations
npm run build:release

# Compile with debug information
npm run build:debug
\`\`\`

### Testing

#### Discover and Run Tests

5IVE CLI discovers test functions from your \`tests/*.v\` files using \`pub test_*\`:

\`\`\`bash
# Run all tests
npm test

# Run with watch mode for continuous testing
5ive test --watch

# Run specific tests by filter
5ive test --filter "test_add"

# Run with verbose output
5ive test --verbose

# Run with JSON output for CI/CD
5ive test --format json

# Run on-chain tests (local/devnet/mainnet)
5ive test --on-chain --target local
5ive test --on-chain --target devnet
5ive test --on-chain --target mainnet --allow-mainnet-tests --max-cost-sol 0.5
\`\`\`

#### Writing Tests

Test functions in your \`.v\` files use the \`pub test_*\` naming convention and include \`@test-params\` comments:

\`\`\`v
// @test-params 10 20 30
pub test_add(a: u64, b: u64) -> u64 {
    return a + b;
}

// @test-params 5 2 10
pub test_multiply(a: u64, b: u64) -> u64 {
    return a * b;
}
\`\`\`

The \`@test-params\` comment specifies inputs. For non-void functions the last value is treated as expected result. The test runner will:
1. Discover test functions automatically
2. Compile the source file
3. Execute with the specified parameters
4. Validate the result matches

For stateful on-chain tests, use companion fixture files (e.g. \`tests/main.test.json\`) to define per-test accounts/parameters.

### Node Client

Use the generated Node starter under \`client/main.ts\` for devnet/mainnet execution:

\`\`\`bash
# Build contract artifact first
npm run build

# Build and run on-chain client
npm run client:build
npm run client:run
\`\`\`

The starter is self-contained (default devnet RPC, generated script-account file, payer auto-loading) and prints signature, \`meta.err\`, and CU.

### Development

\`\`\`bash
# Watch for changes and auto-compile
npm run watch
\`\`\`

### Deployment

\`\`\`bash
# Deploy to devnet
npm run deploy
\`\`\`

## Project Structure

- \`src/\` - 5IVE VM source files (.v)
- \`tests/\` - Test files (.v files with test_* functions)
- \`client/\` - Node TypeScript client starter (FiveProgram + ABI)
- \`build/\` - Compiled bytecode
- \`docs/\` - Documentation
- \`five.toml\` - Project configuration

## Standard Library (Bundled v1)

Projects initialized with \`5ive init\` use compiler-bundled stdlib modules.

Use explicit imports in your modules:

\`\`\`v
use std::builtins;
use std::interfaces::spl_token;
use std::interfaces::system_program;

pub transfer_tokens(
  source: account @mut,
  destination: account @mut,
  authority: account @signer
) {
  spl_token::transfer(source, destination, authority, 1);
}
\`\`\`

See \`docs/STDLIB.md\` for bundled stdlib module details.

### Local Development CLI Note

If your globally installed \`5ive\` binary behaves differently from this repo source, run the local CLI directly:

\`\`\`bash
node ./five-cli/dist/index.js init my-project
\`\`\`

## Multi-File Projects

If your project uses multiple modules with \`use\` or \`import\` statements, 5IVE CLI automatically handles:

\`\`\`bash
# Build from five.toml entry_point using compiler-owned discovery
5ive build
\`\`\`

## Learn More

- [5IVE VM Documentation](https://five-vm.dev)
- [5IVE VM GitHub](https://github.com/five-vm)
- [Multi-File Compilation Guide](./docs/multi-file.md)
- [Examples](./examples)

## License

MIT
`;
}

/**
 * Generate .gitignore content
 */
function generateGitignore(): string {
  return `# Build outputs
build/
*.bin
*.so
*.wasm

# Node.js
node_modules/
npm-debug.log*
yarn-debug.log*
yarn-error.log*

# Environment variables
.env
.env.local

# IDE
.vscode/
.idea/
*.swp
*.swo

# OS
.DS_Store
Thumbs.db

# 5IVE VM
.five/cache/
*.debug.bin
`;
}

/**
 * Generate AGENTS playbooks and write them to project root.
 */
async function generateAgentPlaybooks(projectDir: string): Promise<void> {
  const templateFiles = ['AGENTS.md', 'AGENTS_CHECKLIST.md', 'AGENTS_REFERENCE.md'];

  for (const templateFile of templateFiles) {
    const templateContent = await loadAgentPlaybookTemplate(templateFile);
    await writeFile(join(projectDir, templateFile), templateContent);
  }
}

async function loadAgentPlaybookTemplate(templateFile: string): Promise<string> {
  const __filename = fileURLToPath(import.meta.url);
  const __dirname = dirname(__filename);

  const candidatePaths = [
    // dist runtime
    resolve(__dirname, `../../templates/${templateFile}`),
    // src/test runtime
    resolve(__dirname, `../templates/${templateFile}`),
    // repo-root fallback
    resolve(process.cwd(), `five-cli/templates/${templateFile}`),
  ];

  for (const p of candidatePaths) {
    try {
      const content = await readFile(p, 'utf8');
      if (content.trim().length > 0) {
        return content;
      }
    } catch {
      // try next candidate
    }
  }

  throw new Error(`Failed to load ${templateFile} template from five-cli/templates/${templateFile}`);
}

/**
 * Display success message
 */
function displaySuccessMessage(projectDir: string, projectName: string, options: any): void {
  console.log('\n' + uiSuccess('Project initialized'));
  console.log(`  ${uiColors.info('Generated')} AGENTS.md - Agent operating contract`);
  console.log(`  ${uiColors.info('Generated')} AGENTS_CHECKLIST.md - Step-by-step delivery gates`);
  console.log(`  ${uiColors.info('Generated')} AGENTS_REFERENCE.md - Language and integration reference`);
  console.log('\n' + chalk.bold('Next steps:'));
  
  if (projectDir !== process.cwd()) {
    console.log(`  ${uiColors.info('cd')} ${projectDir}`);
  }
  
  console.log(`  ${uiColors.info('npm install')} - Install dependencies`);
  console.log(`  ${uiColors.info('npm run build')} - Compile the project`);
  console.log(`  ${uiColors.info('npm test')} - Run tests`);
  console.log(`  ${uiColors.info('npm run client:run')} - Run Node client starter`);
  console.log(`  ${uiColors.info('npm run watch')} - Start development mode`);
  
  console.log('\n' + uiColors.muted('Happy coding with 5IVE VM.'));
}
