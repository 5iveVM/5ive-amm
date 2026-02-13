// Init command.

import { writeFile, mkdir, access } from 'fs/promises';
import { join } from 'path';
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
      const projectName = options.name || (args[0] ? args[0] : 'five-project');
      
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
      spinner.succeed('Configuration files generated');

      // Generate agent playbook (always, even with --no-examples)
      spinner.start('Generating AGENTS.md playbook...');
      await generateAgentPlaybook(projectDir, projectName, options.template);
      spinner.succeed('AGENTS.md generated');
      
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
      build: '5ive compile src/**/*.v',
      test: '5ive test',
      deploy: '5ive deploy',
      'build:release': '5ive compile src/**/*.v -O 3',
      'build:debug': '5ive compile src/**/*.v --debug',
      'watch': '5ive compile src/**/*.v --watch'
    },
    devDependencies: {
      'five-cli': '^1.0.0'
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

async function generateExampleFiles(projectDir: string, template: string): Promise<void> {
  // Generate main source file
  const mainFile = getTemplateMainFile(template);
  await writeFile(join(projectDir, 'src/main.v'), mainFile);
  
  // Generate test file
  const testFile = getTemplateTestFile(template);
  await writeFile(join(projectDir, 'tests/main.test.v'), testFile);
  
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
target = "${config.target}"

[optimizations]
enable_compression = ${config.optimizations.enableCompression}
enable_constraint_optimization = ${config.optimizations.enableConstraintOptimization}
optimization_level = "${config.optimizations.optimizationLevel}"

[dependencies]
# Add project dependencies here
# example = { path = "../example" }

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

function getTemplateMainFile(template: string): string {
  const templates: Record<string, string> = {
    basic: `// Basic 5IVE VM Program
script BasicProgram {
    // Program initialization
    init() {
        log("BasicProgram initialized");
    }
    
    // Main program constraints
    constraints {
        // Add your business logic here
        require(true, "Always passes");
    }
}

// Main entry point
instruction main() {
    log("Hello, 5IVE VM!");
    42  // Return value
}

// Example function with parameters
instruction add(a: u64, b: u64) -> u64 {
    a + b
}

#[test]
instruction test_add() {
    let result = add(2, 3);
    assert_eq(result, 5, "Addition should work");
}
`,

    defi: `// DeFi Protocol on 5IVE VM
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

    nft: `// NFT Collection on 5IVE VM
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

    game: `// Game Logic on 5IVE VM
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

    dao: `// DAO Governance on 5IVE VM
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
// Use @test-params to specify function parameters for testing
// Format: @test-params <param1> <param2> ... <expected_result>

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

5IVE CLI automatically discovers test functions from your \`.v\` files:

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

The \`@test-params\` comment specifies the parameters to pass and expected result. The test runner will:
1. Discover test functions automatically
2. Compile the source file
3. Execute with the specified parameters
4. Validate the result matches

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
- \`build/\` - Compiled bytecode
- \`docs/\` - Documentation
- \`five.toml\` - Project configuration

## Multi-File Projects

If your project uses multiple modules with \`use\` or \`import\` statements, 5IVE CLI automatically handles:

\`\`\`bash
# Automatic discovery of imported modules
5ive compile src/main.v --auto-discover

# Or use the build command which respects five.toml configuration
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
 * Generate AGENTS.md playbook content and write it to project root.
 */
async function generateAgentPlaybook(
  projectDir: string,
  projectName: string,
  template: string
): Promise<void> {
  const playbook = `# AGENTS.md - Complete 5IVE DSL Agent Bootstrap

This file is designed for agents with zero prior 5IVE knowledge.
It is intentionally self-contained and should be treated as the minimum operating spec for building and deploying 5IVE bytecode safely.

## 1. What 5IVE Is

5IVE is a DSL and toolchain for building compact Solana-executed program logic compiled into 5IVE bytecode.

Core outputs and surfaces:
1. Source files: \`.v\`
2. Canonical artifact: \`.five\` (bytecode + ABI)
3. CLI: \`@5ive-tech/cli\` (\`5ive\` canonical command, \`five\` alias also works)
4. SDK: \`@5ive-tech/sdk\` (\`FiveSDK\`, \`FiveProgram\`)

Project context for this repository:
- Name: ${projectName}
- Template: ${template}

## 2. Source of Truth Policy

When references conflict, resolve in this order:
1. CLI/SDK/compiler source code
2. Package manifests and command definitions
3. README/docs/examples

Do not assume older docs are correct without verifying against active command implementations.

## 3. Non-Negotiable Workflow

1. Inspect \`five.toml\` first.
2. Compile to \`.five\` artifact.
3. Run local/runtime tests.
4. Deploy with explicit target and program ID resolution path.
5. Execute and verify confirmed transaction metadata (\`meta.err == null\`).
6. Record signatures + compute units.

## 4. 5IVE DSL Syntax and Semantics (Cold-Start Primer)

### 4.1 Top-level structure
Current examples and grammar support top-level declarations directly.

\`\`\`five
account Counter {
    value: u64;
    authority: pubkey;
}

pub increment(counter: Counter @mut, authority: account @signer) {
    require(counter.authority == authority.key);
    counter.value = counter.value + 1;
}
\`\`\`

Notes:
1. Legacy wrapper forms may exist in old examples; prefer direct top-level declarations.
2. Keep one clear entrypoint file for project builds (\`five.toml\` \`entry_point\`).

### 4.2 Core declarations
1. Accounts:
\`\`\`five
account Position {
    owner: pubkey;
    amount: u64;
}
\`\`\`
2. Functions:
\`\`\`five
pub add(a: u64, b: u64) -> u64 {
    return a + b;
}
\`\`\`
3. Init block (used in many examples):
\`\`\`five
init {
    // initial setup
}
\`\`\`

### 4.3 Types
Commonly used types from docs/examples:
1. Unsigned ints: \`u8..u128\`
2. Signed ints: \`i8..i64\`
3. \`bool\`
4. \`pubkey\`
5. Strings with sizing in account fields: \`string<N>\`
6. Fixed arrays: \`[T; N]\`
7. Optional account fields: \`field?: type\`
8. Option/Result in signatures in advanced examples: \`Option<T>\`, \`Result<T,E>\`

Use conservative, template-proven type patterns for production paths.

### 4.4 Expressions and control flow
Supported in examples:
1. Arithmetic and comparisons
2. Boolean logic
3. \`if\` and nested conditionals
4. \`while\`
5. Function calls and account field access

Example:
\`\`\`five
pub accumulate(limit: u64) -> u64 {
    let mut i: u64 = 0;
    let mut total: u64 = 0;
    while (i < limit) {
        total = total + i;
        i = i + 1;
    }
    return total;
}
\`\`\`

### 4.5 Guards and validation
Use \`require(...)\` aggressively for invariant protection.

\`\`\`five
require(amount > 0);
require(vault.authority == authority.key);
\`\`\`

### 4.6 Account parameters and constraints
Canonical constraint patterns:
1. \`@mut\` mutable account
2. \`@signer\` required signer
3. \`@init\` initialize account
4. Extended patterns in templates:
   - \`@init(payer=..., space=..., seeds=[...])\`
   - \`@has(field)\` ownership/authority relation checks

Example:
\`\`\`five
pub init_counter(
    counter: Counter @mut @init(payer=owner, space=56, seeds=["counter", owner.key]),
    owner: account @signer
) {
    counter.value = 0;
    counter.authority = owner.key;
}
\`\`\`

### 4.7 External bytecode imports
5IVE supports import-style external calls to deployed bytecode accounts.

\`\`\`five
use "11111111111111111111111111111111"::{transfer};

pub settle(from: account @mut, to: account @mut, owner: account @signer) {
    transfer(from, to, owner, 50);
}
\`\`\`

### 4.8 CPI interfaces
Define external program interfaces explicitly:

\`\`\`five
interface SPLToken @program("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA") @serializer(bincode) {
    transfer @discriminator(3) (
        from: account,
        to: account,
        authority: account,
        amount: u64
    );
}

pub cpi_transfer(from: account @mut, to: account @mut, authority: account @signer) {
    SPLToken.transfer(from, to, authority, 50);
}
\`\`\`

Critical CPI rules:
1. Always set \`@program(...)\`.
2. Always set \`@serializer(...)\` explicitly.
3. Always set \`@discriminator(...)\` explicitly.
4. Keep account ordering deterministic.

### 4.9 Serializer conflict handling (important)
Docs in this repo historically conflict on default serializer (\`borsh\` vs \`bincode\`).
Canonical rule for agents:
1. Never rely on default serializer.
2. Always specify serializer explicitly in each interface.
3. Match serializer/discriminator to target program spec.

### 4.10 DSL safety baseline
For each state-mutating function, include:
1. authority check
2. value/range check
3. state-transition check
4. arithmetic safety check (overflow/underflow-aware patterns)

## 5. Project Structure and Build Model

Typical project layout:
1. \`src/\` DSL source
2. \`tests/\` test scripts
3. \`build/\` compiled artifacts
4. \`five.toml\` project config

Multi-file model:
1. Set \`entry_point\` in \`five.toml\`.
2. If using module list, keep stable ordering.
3. Ensure all imported modules are part of build context.

## 6. CLI Canonical Usage

### 6.1 Install and identity
\`\`\`bash
npm install -g @5ive-tech/cli
5ive --version
\`\`\`

### 6.2 Initialize
\`\`\`bash
5ive init my-program
cd my-program
\`\`\`

### 6.3 Compile
\`\`\`bash
5ive compile src/main.v -o build/main.five
# or project-aware
5ive build
\`\`\`

### 6.4 Local execute
\`\`\`bash
5ive execute build/main.five --local -f 0
\`\`\`

### 6.5 Configure devnet
\`\`\`bash
5ive config init
5ive config set --target devnet
5ive config set --keypair ~/.config/solana/id.json
5ive config set --program-id <FIVE_VM_PROGRAM_ID> --target devnet
\`\`\`

### 6.6 Deploy and execute on-chain
\`\`\`bash
5ive deploy build/main.five --target devnet
5ive execute build/main.five --target devnet -f 0
\`\`\`

### 6.7 Advanced deploy modes
\`\`\`bash
5ive deploy build/main.five --target devnet --optimized --progress
5ive deploy build/main.five --target devnet --force-chunked --chunk-size 900
5ive deploy build/main.five --target devnet --dry-run --format json
\`\`\`

### 6.8 Tests
\`\`\`bash
5ive test --sdk-runner
5ive test tests/ --on-chain --target devnet
5ive test --sdk-runner --format json
\`\`\`

## 7. Program ID and Target Resolution

For on-chain commands (\`deploy\`, \`execute\`, \`namespace\`) resolve program ID in this order:
1. \`--program-id\`
2. \`five.toml [deploy].program_id\`
3. \`5ive config\` stored value for current target
4. \`FIVE_PROGRAM_ID\`

If unresolved, fail fast and do not continue.

## 8. SDK Canonical Usage

### 8.1 Load artifact
\`\`\`ts
import fs from "fs";
import { FiveSDK } from "@5ive-tech/sdk";

const fiveFileText = fs.readFileSync("build/main.five", "utf8");
const { abi } = await FiveSDK.loadFiveFile(fiveFileText);
\`\`\`

### 8.2 Program client
\`\`\`ts
import { FiveProgram } from "@5ive-tech/sdk";

const program = FiveProgram.fromABI("<SCRIPT_ACCOUNT>", abi, {
  fiveVMProgramId: "<FIVE_VM_PROGRAM_ID>",
  vmStateAccount: "<VM_STATE_ACCOUNT>",
  feeReceiverAccount: "<FEE_RECEIVER_ACCOUNT>",
});
\`\`\`

### 8.3 Instruction build + send
1. Build with \`.function().accounts().args().instruction()\`.
2. Convert to \`TransactionInstruction\`.
3. Send with preflight.
4. Fetch confirmed transaction.
5. Assert \`meta.err == null\`.
6. Record \`meta.computeUnitsConsumed\`.

### 8.4 SDK program ID defaults
Precedence in SDK paths:
1. explicit \`fiveVMProgramId\`
2. \`FiveSDK.setDefaultProgramId(...)\`
3. \`FIVE_PROGRAM_ID\`
4. released package baked default (if set)

## 9. Frontend Integration Baseline

1. Build instructions through SDK (\`FiveProgram\`) instead of custom serializers.
2. Keep network selection explicit (\`localnet\`, \`devnet\`, \`mainnet\`).
3. Surface signatures, errors, and CU metrics in UI.
4. For editor workflows, use LSP-backed diagnostics/completion features where available.

## 10. Design Pattern Mapping (for Complex Programs)

### 10.1 Vault
- Accounts: vault state, authority signer, source/destination token accounts
- Invariants: authority-only withdrawals, no negative balances

### 10.2 Escrow
- Accounts: escrow state, counterparties, settlement accounts
- Invariants: valid lifecycle transitions, no double settlement

### 10.3 Token/mint authority
- Accounts: mint, token accounts, authorities/delegates
- Invariants: supply accounting, authority checks, freeze/delegate behavior

### 10.4 AMM/orderbook
- Accounts: pool/book state, user positions, fee accounts
- Invariants: conservation, deterministic settlement, fee accounting

### 10.5 Lending/perps/stablecoin
- Accounts: collateral/debt/position state, oracle/risk accounts
- Invariants: collateral thresholds, liquidation boundaries

## 11. Testing Strategy

Execution order:
1. Runtime harness (validator-free) where available
2. Local CLI/SDK tests
3. On-chain integration tests

Always include:
1. happy path
2. auth failure path
3. value-range failure path
4. state transition failure path
5. CU regression capture for critical instructions

## 12. Mainnet Safety Policy

Never deploy mainnet blindly.

Required preflight gates:
1. Artifact hash freeze (\`.five\` file chosen and immutable)
2. Config lock (target, RPC, program ID, keypair source)
3. Key custody validation
4. Dry-run/simulate path complete
5. Rollback/containment plan defined

Post-deploy requirements:
1. smoke execute
2. confirmed transaction validation
3. CU baseline capture
4. incident path if unexpected errors appear

## 13. Common Failures and Fixes

1. \`No program ID resolved for Five VM\`:
   - Set one via \`--program-id\`, config, or \`FIVE_PROGRAM_ID\`.
2. \`Function '<name>' not found in ABI\`:
   - Use exact ABI name (including namespace prefixes).
3. \`Missing required account\` / \`Missing required argument\`:
   - satisfy all \`.accounts(...)\` and \`.args(...)\` fields.
4. owner/program mismatch:
   - check target program ID and deployed account ownership.
5. CPI serialization mismatch:
   - ensure explicit \`@serializer(...)\` and correct discriminator format.

## 14. Definition of Done

A task is complete only when:
1. Program compiles to \`.five\`.
2. Tests pass with evidence.
3. Deployment is confirmed (if requested).
4. Execution is confirmed and \`meta.err == null\` (if requested).
5. Signatures + CU data are recorded.
6. SDK/frontend integration snippet is provided if integration is in scope.

## 15. Agent Behavior Rules

1. Prefer minimal, reproducible command paths.
2. Do not skip verification after sending transactions.
3. Do not assume defaults for critical deploy/CPI parameters.
4. Keep all outputs deterministic and auditable.
5. If uncertain, inspect compiler/CLI source before making assumptions.
`;

  await writeFile(join(projectDir, 'AGENTS.md'), playbook);
}

/**
 * Display success message
 */
function displaySuccessMessage(projectDir: string, projectName: string, options: any): void {
  console.log('\n' + uiSuccess('Project initialized'));
  console.log(`  ${uiColors.info('Generated')} AGENTS.md - Agent playbook for build/deploy/test workflow`);
  console.log('\n' + chalk.bold('Next steps:'));
  
  if (projectDir !== process.cwd()) {
    console.log(`  ${uiColors.info('cd')} ${projectDir}`);
  }
  
  console.log(`  ${uiColors.info('npm install')} - Install dependencies`);
  console.log(`  ${uiColors.info('npm run build')} - Compile the project`);
  console.log(`  ${uiColors.info('npm test')} - Run tests`);
  console.log(`  ${uiColors.info('npm run watch')} - Start development mode`);
  
  console.log('\n' + uiColors.muted('Happy coding with 5IVE VM.'));
}
