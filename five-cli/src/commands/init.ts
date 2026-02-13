// Init command.

import { writeFile, mkdir, access, readFile } from 'fs/promises';
import { join, resolve, dirname } from 'path';
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
      await generateAgentPlaybook(projectDir);
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
  projectDir: string
): Promise<void> {
  const playbookTemplate = await loadAgentPlaybookTemplate();
  await writeFile(join(projectDir, 'AGENTS.md'), playbookTemplate);
}

async function loadAgentPlaybookTemplate(): Promise<string> {
  const __filename = fileURLToPath(import.meta.url);
  const __dirname = dirname(__filename);

  const candidatePaths = [
    // dist runtime
    resolve(__dirname, '../../templates/AGENTS.md'),
    // src/test runtime
    resolve(__dirname, '../templates/AGENTS.md'),
    // repo-root fallback
    resolve(process.cwd(), 'five-cli/templates/AGENTS.md'),
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

  throw new Error('Failed to load AGENTS.md template from five-cli/templates/AGENTS.md');
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
