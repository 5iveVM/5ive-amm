// Analyze command.

import { readFile } from 'fs/promises';
import { extname } from 'path';
import { section } from '../utils/cli-ui.js';
import ora from 'ora';

import {
  CommandDefinition,
  CommandContext,
  BytecodeAnalysis,
  CLIOptions
} from '../types.js';
import { FiveCompilerWasm } from '../wasm/compiler.js';
import { FiveFileManager } from '../utils/FiveFileManager.js';

export const analyzeCommand: CommandDefinition = {
  name: 'analyze',
  description: 'Analyze Five VM bytecode for optimization and security',
  aliases: ['analysis', 'inspect'],
  
  options: [
    {
      flags: '--security',
      description: 'Perform security analysis',
      defaultValue: false
    },
    {
      flags: '--performance',
      description: 'Analyze performance characteristics',
      defaultValue: false
    },
    {
      flags: '--optimization',
      description: 'Show optimization opportunities',
      defaultValue: false
    },
    {
      flags: '--all',
      description: 'Perform all types of analysis',
      defaultValue: false
    },
    {
      flags: '--format <format>',
      description: 'Output format',
      choices: ['text', 'json', 'html'],
      defaultValue: 'text'
    },
    {
      flags: '--output <file>',
      description: 'Save analysis report to file',
      required: false
    },
    {
      flags: '--verbose',
      description: 'Show detailed analysis information',
      defaultValue: false
    }
  ],

  arguments: [
    {
      name: 'bytecode',
      description: 'Five VM bytecode file (.bin)',
      required: true
    }
  ],

  examples: [
    {
      command: 'five analyze program.bin',
      description: 'Basic bytecode analysis'
    },
    {
      command: 'five analyze program.bin --all --format json',
      description: 'Complete analysis with JSON output'
    },
    {
      command: 'five analyze program.bin --security --output report.html',
      description: 'Security analysis with HTML report'
    },
    {
      command: 'five analyze program.bin --performance --verbose',
      description: 'Detailed performance analysis'
    }
  ],

  handler: async (args: string[], options: any, context: CommandContext): Promise<void> => {
    const { logger, wasmManager } = context;
    
    try {
      // Initialize WASM compiler for analysis
      const spinner = ora('Initializing Five analyzer...').start();
      
      const compiler = new FiveCompilerWasm(logger);
      await compiler.initialize();
      
      spinner.succeed('Five analyzer initialized');

      // Load bytecode
      const bytecodeFile = args[0];
      
      // Load file using centralized manager
      const fileManager = FiveFileManager.getInstance();
      const loadedFile = await fileManager.loadFile(bytecodeFile, { 
        validateFormat: true 
      });
      
      logger.info(`Analyzing ${loadedFile.format.toUpperCase()} file: ${loadedFile.bytecode.length} bytes from ${bytecodeFile}`);
      
      // Show ABI info if available
      if (loadedFile.abi) {
        const functionCount = Object.keys(loadedFile.abi.functions || {}).length;
        logger.info(`Functions to analyze: ${functionCount}`);
      }

      // Perform analysis
      spinner.start('Analyzing bytecode...');
      const analysis = await compiler.analyzeBytecode(loadedFile.bytecode);
      spinner.succeed('Analysis completed');

      // Display results
      await displayAnalysisResults(analysis, options, logger);

    } catch (error) {
      logger.error('Analysis failed:', error);
      throw error;
    }
  }
};

/**
 * Display analysis results in specified format
 */
async function displayAnalysisResults(
  analysis: BytecodeAnalysis,
  options: any,
  logger: any
): Promise<void> {
  if (options.format === 'json') {
    const output = JSON.stringify(analysis, null, 2);
    
    if (options.output) {
      const { writeFile } = await import('fs/promises');
      await writeFile(options.output, output);
      logger.info(`Analysis saved to ${options.output}`);
    } else {
      console.log(output);
    }
    return;
  }

  if (options.format === 'html') {
    const htmlReport = generateHtmlReport(analysis);
    
    if (options.output) {
      const { writeFile } = await import('fs/promises');
      await writeFile(options.output, htmlReport);
      logger.info(`HTML report saved to ${options.output}`);
    } else {
      console.log(htmlReport);
    }
    return;
  }

  // Text format output
  displayTextAnalysis(analysis, options);
}

/**
 * Display analysis in text format
 */
function displayTextAnalysis(analysis: BytecodeAnalysis, options: any): void {
  console.log('\n' + section('Five VM Bytecode Analysis'));
  
  // Basic Information
  console.log('\n' + section('Basic Information'));
  console.log(`  Instructions: ${analysis.instructionCount}`);
  console.log(`  Functions: ${analysis.functionCount}`);
  console.log(`  Jump Targets: ${analysis.jumpTargets.length}`);
  
  // Complexity Metrics
  if (analysis.complexity) {
    console.log('\n' + section('Complexity Metrics'));
    console.log(`  Cyclomatic Complexity: ${analysis.complexity.cyclomaticComplexity}`);
    console.log(`  Nesting Depth: ${analysis.complexity.nestingDepth}`);
    console.log(`  Halstead Complexity: ${analysis.complexity.halsteadComplexity}`);
    console.log(`  Maintainability Index: ${analysis.complexity.maintainabilityIndex}`);
  }
  
  // Call Graph
  if (options.verbose && analysis.callGraph.length > 0) {
    console.log('\n' + section('Call Graph'));
    for (const node of analysis.callGraph.slice(0, 10)) {
      console.log(`  ${node.functionName}:`);
      console.log(`    Calls: ${node.callsTo.join(', ') || 'none'}`);
      console.log(`    Called by: ${node.calledBy.join(', ') || 'none'}`);
      console.log(`    Instructions: ${node.instructionCount}`);
      console.log(`    Complexity: ${node.complexity}`);
    }
    
    if (analysis.callGraph.length > 10) {
      console.log(`    ... and ${analysis.callGraph.length - 10} more functions`);
    }
  }
  
  // Optimization Opportunities
  if ((options.optimization || options.all) && analysis.optimizationOpportunities.length > 0) {
    console.log('\n' + section('Optimization Opportunities'));
    
    const grouped = groupBy(analysis.optimizationOpportunities, 'priority');
    
    for (const priority of ['high', 'medium', 'low']) {
      const opportunities = grouped[priority] || [];
      if (opportunities.length > 0) {
        console.log(`\n  ${priority.toUpperCase()} Priority:`);
        
        for (const opp of opportunities.slice(0, 5)) {
          console.log(`    • ${opp.description}`);
          console.log(`      Location: ${opp.location}`);
          console.log(`      Estimated improvement: ${opp.estimatedImprovement}`);
          if (options.verbose) {
            console.log(`      Type: ${opp.type}`);
          }
        }
        
        if (opportunities.length > 5) {
          console.log(`      ... and ${opportunities.length - 5} more`);
        }
      }
    }
  }
  
  // Security Issues
  if ((options.security || options.all) && analysis.securityIssues.length > 0) {
    console.log('\n' + section('Security Analysis'));
    
    const grouped = groupBy(analysis.securityIssues, 'severity');
    
    for (const severity of ['critical', 'high', 'medium', 'low']) {
      const issues = grouped[severity] || [];
      if (issues.length > 0) {
        console.log(`\n  ${severity.toUpperCase()} Severity:`);
        
        for (const issue of issues) {
          console.log(`    ${getSecurityIcon(severity)} ${issue.description}`);
          console.log(`      Location: ${issue.location}`);
          console.log(`      Category: ${issue.category}`);
          if (options.verbose) {
            console.log(`      Recommendation: ${issue.recommendation}`);
          }
        }
      }
    }
  }
  
  // Performance Analysis
  if ((options.performance || options.all)) {
    console.log('\n' + section('Performance Analysis'));
    
    // Estimate compute units
    const estimatedCU = estimateComputeUnits(analysis);
    console.log(`  Estimated Compute Units: ${estimatedCU}`);
    
    // Memory usage estimation
    const memoryEstimate = estimateMemoryUsage(analysis);
    console.log(`  Estimated Memory Usage: ${memoryEstimate} bytes`);
    
    // Performance characteristics
    const characteristics = analyzePerformanceCharacteristics(analysis);
    console.log(`  Performance Profile: ${characteristics.profile}`);
    console.log(`  Bottlenecks: ${characteristics.bottlenecks.join(', ') || 'none detected'}`);
    
    if (options.verbose && characteristics.hotspots.length > 0) {
      console.log('\n  Performance Hotspots:');
      for (const hotspot of characteristics.hotspots.slice(0, 5)) {
        console.log(`    • ${hotspot.location}: ${hotspot.impact}`);
      }
    }
  }
  
  // Summary
  console.log('\n' + section('Summary'));
  const score = calculateOverallScore(analysis);
  console.log(`  Overall Score: ${score}/100`);
  
  const recommendations = generateRecommendations(analysis);
  if (recommendations.length > 0) {
    console.log('\n' + section('Recommendations'));
    for (const rec of recommendations.slice(0, 3)) {
      console.log(`  • ${rec}`);
    }
  }
}

/**
 * Generate HTML report
 */
function generateHtmlReport(analysis: BytecodeAnalysis): string {
  return `<!DOCTYPE html>
<html>
<head>
    <title>Five VM Bytecode Analysis Report</title>
    <style>
        body { font-family: Arial, sans-serif; margin: 40px; }
        .header { background: #f4f4f4; padding: 20px; border-radius: 5px; }
        .section { margin: 20px 0; }
        .metric { display: inline-block; margin: 10px; padding: 10px; background: #e9e9e9; border-radius: 3px; }
        .high { color: #d32f2f; }
        .medium { color: #f57f17; }
        .low { color: #388e3c; }
        .critical { color: #b71c1c; font-weight: bold; }
        table { border-collapse: collapse; width: 100%; }
        th, td { border: 1px solid #ddd; padding: 8px; text-align: left; }
        th { background-color: #f2f2f2; }
    </style>
</head>
<body>
    <div class="header">
        <h1>Five VM Bytecode Analysis Report</h1>
        <p>Generated on ${new Date().toISOString()}</p>
    </div>
    
    <div class="section">
        <h2>Basic Information</h2>
        <div class="metric">Instructions: ${analysis.instructionCount}</div>
        <div class="metric">Functions: ${analysis.functionCount}</div>
        <div class="metric">Jump Targets: ${analysis.jumpTargets.length}</div>
    </div>
    
    ${analysis.complexity ? `
    <div class="section">
        <h2>Complexity Metrics</h2>
        <div class="metric">Cyclomatic: ${analysis.complexity.cyclomaticComplexity}</div>
        <div class="metric">Nesting Depth: ${analysis.complexity.nestingDepth}</div>
        <div class="metric">Halstead: ${analysis.complexity.halsteadComplexity}</div>
        <div class="metric">Maintainability: ${analysis.complexity.maintainabilityIndex}</div>
    </div>
    ` : ''}
    
    ${analysis.optimizationOpportunities.length > 0 ? `
    <div class="section">
        <h2>Optimization Opportunities</h2>
        <table>
            <tr><th>Priority</th><th>Type</th><th>Description</th><th>Location</th><th>Improvement</th></tr>
            ${analysis.optimizationOpportunities.map(opp => `
                <tr>
                    <td class="${opp.priority}">${opp.priority}</td>
                    <td>${opp.type}</td>
                    <td>${opp.description}</td>
                    <td>${opp.location}</td>
                    <td>${opp.estimatedImprovement}</td>
                </tr>
            `).join('')}
        </table>
    </div>
    ` : ''}
    
    ${analysis.securityIssues.length > 0 ? `
    <div class="section">
        <h2>Security Issues</h2>
        <table>
            <tr><th>Severity</th><th>Category</th><th>Description</th><th>Location</th></tr>
            ${analysis.securityIssues.map(issue => `
                <tr>
                    <td class="${issue.severity}">${issue.severity}</td>
                    <td>${issue.category}</td>
                    <td>${issue.description}</td>
                    <td>${issue.location}</td>
                </tr>
            `).join('')}
        </table>
    </div>
    ` : ''}
    
</body>
</html>`;
}

/**
 * Utility functions
 */
function groupBy<T>(array: T[], key: keyof T): Record<string, T[]> {
  return array.reduce((groups, item) => {
    const group = (groups[item[key] as string] = groups[item[key] as string] || []);
    group.push(item);
    return groups;
  }, {} as Record<string, T[]>);
}

function getSecurityIcon(severity: string): string {
  switch (severity) {
    case 'critical': return 'CRIT';
    case 'high': return 'HIGH';
    case 'medium': return 'MED';
    case 'low': return 'LOW';
    default: return '-';
  }
}

function estimateComputeUnits(analysis: BytecodeAnalysis): number {
  // Simple estimation based on instruction count and complexity
  const baseUnits = analysis.instructionCount * 2;
  const complexityMultiplier = analysis.complexity ? 
    1 + (analysis.complexity.cyclomaticComplexity * 0.1) : 1;
  
  return Math.round(baseUnits * complexityMultiplier);
}

function estimateMemoryUsage(analysis: BytecodeAnalysis): number {
  // Estimate based on instruction count and function count
  const instructionMemory = analysis.instructionCount * 4; // 4 bytes per instruction
  const functionMemory = analysis.functionCount * 32; // 32 bytes per function header
  const stackMemory = 1024; // Base stack allocation
  
  return instructionMemory + functionMemory + stackMemory;
}

function analyzePerformanceCharacteristics(analysis: BytecodeAnalysis): {
  profile: string;
  bottlenecks: string[];
  hotspots: Array<{ location: string; impact: string }>;
} {
  const profile = analysis.complexity?.cyclomaticComplexity > 10 ? 'Complex' : 
                  analysis.instructionCount > 1000 ? 'Large' : 'Simple';
  
  const bottlenecks: string[] = [];
  const hotspots: Array<{ location: string; impact: string }> = [];
  
  // Analyze call graph for bottlenecks
  for (const node of analysis.callGraph) {
    if (node.complexity > 5) {
      bottlenecks.push(`Complex function: ${node.functionName}`);
      hotspots.push({
        location: node.functionName,
        impact: `High complexity (${node.complexity})`
      });
    }
    
    if (node.instructionCount > 100) {
      bottlenecks.push(`Large function: ${node.functionName}`);
      hotspots.push({
        location: node.functionName,
        impact: `Many instructions (${node.instructionCount})`
      });
    }
  }
  
  return { profile, bottlenecks, hotspots };
}

function calculateOverallScore(analysis: BytecodeAnalysis): number {
  let score = 100;
  
  // Deduct for complexity
  if (analysis.complexity) {
    if (analysis.complexity.cyclomaticComplexity > 10) score -= 20;
    if (analysis.complexity.nestingDepth > 5) score -= 15;
    if (analysis.complexity.maintainabilityIndex < 50) score -= 25;
  }
  
  // Deduct for security issues
  for (const issue of analysis.securityIssues) {
    switch (issue.severity) {
      case 'critical': score -= 30; break;
      case 'high': score -= 20; break;
      case 'medium': score -= 10; break;
      case 'low': score -= 5; break;
    }
  }
  
  // Deduct for optimization opportunities
  const highPriorityOptimizations = analysis.optimizationOpportunities
    .filter(opp => opp.priority === 'high').length;
  score -= highPriorityOptimizations * 5;
  
  return Math.max(0, score);
}

function generateRecommendations(analysis: BytecodeAnalysis): string[] {
  const recommendations: string[] = [];
  
  if (analysis.complexity?.cyclomaticComplexity > 10) {
    recommendations.push('Consider refactoring complex functions to reduce cyclomatic complexity');
  }
  
  const criticalSecurity = analysis.securityIssues.filter(issue => issue.severity === 'critical');
  if (criticalSecurity.length > 0) {
    recommendations.push('Address critical security issues immediately');
  }
  
  const highPriorityOpts = analysis.optimizationOpportunities.filter(opp => opp.priority === 'high');
  if (highPriorityOpts.length > 0) {
    recommendations.push('Implement high-priority optimizations for better performance');
  }
  
  if (analysis.instructionCount > 1000) {
    recommendations.push('Consider code splitting for large programs');
  }
  
  return recommendations;
}
