#!/usr/bin/env python3
"""
Rust Binary Output Repository Mapping System
Maps Rust intelligence system outputs to repository structure and Hermes profile organization

Author: Christian Mahardhika
Purpose: Systematic integration between Rust backend, repository, and profile structure
"""

import os
import json
import shutil
from datetime import datetime
from pathlib import Path
import logging

logging.basicConfig(level=logging.INFO, format='%(asctime)s - %(levelname)s - %(message)s')
logger = logging.getLogger(__name__)

class RustOutputMapper:
    """Maps Rust intelligence system outputs to repository and profile structure"""
    
    def __init__(self):
        # Base paths
        self.rust_system_path = Path("/home/ctianm/intelligence-system/intelligence-system-rust")
        self.rust_binary = self.rust_system_path / "target/release/intelligence-system-rust"
        self.advanced_system_path = Path("/home/ctianm/advanced-intelligence-system")
        self.hermes_profile_path = Path("/home/ctianm/.hermes/profiles/social-politic-lab")
        self.repository_path = Path("/home/ctianm/hermes-data-pipeline")
        
        # Output mapping configuration
        self.mapping_config = {
            # Rust binary outputs
            "rust_outputs": {
                "api_data": "/tmp/intelligence-system-rust-api-*.json",
                "news_intelligence": "/tmp/news-intelligence-*.json", 
                "commodity_data": "/tmp/commodity-data-*.json",
                "social_intelligence": "/tmp/social-intelligence-*.json",
                "correlation_analysis": "/tmp/correlation-analysis-*.json"
            },
            
            # Repository structure mapping
            "repository_structure": {
                "data": "data/",
                "outputs": "outputs/",
                "intelligence": "intelligence-reports/",
                "commodities": "market-data/",
                "social": "social-intelligence/",
                "correlations": "correlation-analysis/",
                "logs": "logs/"
            },
            
            # Hermes profile organization
            "profile_structure": {
                "intelligence_cache": "intelligence/",
                "market_data": "market-data/",
                "social_reports": "social-intelligence/", 
                "correlation_reports": "correlations/",
                "system_logs": "logs/",
                "backup_data": "backups/"
            },
            
            # Topic delivery mapping
            "topic_mapping": {
                "financial_intelligence": "telegram:pagupon finance",
                "social_intelligence": "origin",  # HOME topic
                "system_monitoring": "origin",
                "correlation_analysis": "telegram:pagupon finance"
            }
        }
    
    def check_rust_binary_status(self) -> dict:
        """Check Rust binary compilation and runtime status"""
        status = {
            "binary_exists": self.rust_binary.exists(),
            "binary_size": 0,
            "last_modified": None,
            "is_running": False,
            "process_id": None
        }
        
        if status["binary_exists"]:
            stat = self.rust_binary.stat()
            status["binary_size"] = stat.st_size
            status["last_modified"] = datetime.fromtimestamp(stat.st_mtime).isoformat()
            
            # Check if process is running
            import subprocess
            try:
                result = subprocess.run(
                    ["pgrep", "-f", "intelligence-system-rust"],
                    capture_output=True, text=True
                )
                if result.stdout.strip():
                    status["is_running"] = True
                    status["process_id"] = result.stdout.strip().split('\n')[0]
            except Exception as e:
                logger.warning(f"Could not check process status: {e}")
        
        return status
    
    def discover_rust_outputs(self) -> dict:
        """Discover all Rust binary output files"""
        outputs = {
            "api_outputs": [],
            "data_files": [],
            "log_files": [],
            "temp_files": [],
            "advanced_system_outputs": []
        }
        
        # Check various output locations
        output_locations = [
            "/tmp/",
            str(self.rust_system_path),
            str(self.advanced_system_path),
            str(self.hermes_profile_path / "cron/output"),
            "/home/ctianm/"
        ]
        
        for location in output_locations:
            if os.path.exists(location):
                for root, dirs, files in os.walk(location):
                    for file in files:
                        file_path = os.path.join(root, file)
                        
                        # Categorize outputs
                        if any(pattern in file for pattern in ["intelligence", "commodity", "social", "correlation"]):
                            if file.endswith('.json'):
                                outputs["data_files"].append(file_path)
                            elif file.endswith('.log'):
                                outputs["log_files"].append(file_path)
                        
                        if "advanced-intelligence" in file or "enhanced_social" in file:
                            outputs["advanced_system_outputs"].append(file_path)
                        
                        if file.startswith("api_") or "localhost" in file:
                            outputs["api_outputs"].append(file_path)
                        
                        if file.startswith("tmp_") or "temp" in file:
                            outputs["temp_files"].append(file_path)
        
        return outputs
    
    def create_repository_mapping(self) -> dict:
        """Create systematic mapping to repository structure"""
        mapping = {}
        
        # Ensure repository structure exists
        for category, path in self.mapping_config["repository_structure"].items():
            full_path = self.repository_path / path
            full_path.mkdir(parents=True, exist_ok=True)
            mapping[category] = str(full_path)
        
        logger.info(f"Repository structure created at: {self.repository_path}")
        return mapping
    
    def create_profile_mapping(self) -> dict:
        """Create systematic mapping to Hermes profile structure"""
        mapping = {}
        
        # Ensure profile structure exists
        for category, path in self.mapping_config["profile_structure"].items():
            full_path = self.hermes_profile_path / path
            full_path.mkdir(parents=True, exist_ok=True)
            mapping[category] = str(full_path)
        
        logger.info(f"Profile structure created at: {self.hermes_profile_path}")
        return mapping
    
    def map_outputs_to_structure(self, outputs: dict, repo_mapping: dict, profile_mapping: dict) -> dict:
        """Map discovered outputs to repository and profile structures"""
        mapping_results = {
            "repository_mappings": {},
            "profile_mappings": {},
            "topic_deliveries": {},
            "files_processed": 0
        }
        
        # Process data files
        for file_path in outputs["data_files"]:
            filename = os.path.basename(file_path)
            
            # Determine category and target locations
            if "commodity" in filename or "market" in filename:
                repo_target = repo_mapping["commodities"]
                profile_target = profile_mapping["market_data"]
                topic = self.mapping_config["topic_mapping"]["financial_intelligence"]
            elif "social" in filename:
                repo_target = repo_mapping["social"]
                profile_target = profile_mapping["social_reports"]
                topic = self.mapping_config["topic_mapping"]["social_intelligence"]
            elif "correlation" in filename:
                repo_target = repo_mapping["correlations"]
                profile_target = profile_mapping["correlation_reports"]
                topic = self.mapping_config["topic_mapping"]["correlation_analysis"]
            else:
                repo_target = repo_mapping["intelligence"]
                profile_target = profile_mapping["intelligence_cache"]
                topic = self.mapping_config["topic_mapping"]["system_monitoring"]
            
            # Store mapping information
            mapping_results["repository_mappings"][filename] = {
                "source": file_path,
                "target": os.path.join(repo_target, filename),
                "category": "data"
            }
            
            mapping_results["profile_mappings"][filename] = {
                "source": file_path,
                "target": os.path.join(profile_target, filename),
                "category": "data"
            }
            
            mapping_results["topic_deliveries"][filename] = topic
            mapping_results["files_processed"] += 1
        
        return mapping_results
    
    def execute_mapping(self, dry_run: bool = True) -> dict:
        """Execute the mapping process"""
        logger.info("🎯 Starting Rust Output Repository Mapping...")
        
        # Check system status
        rust_status = self.check_rust_binary_status()
        logger.info(f"Rust binary status: {rust_status}")
        
        # Discover outputs
        outputs = self.discover_rust_outputs()
        logger.info(f"Discovered outputs: {sum(len(v) for v in outputs.values())} files")
        
        # Create structure mappings
        repo_mapping = self.create_repository_mapping()
        profile_mapping = self.create_profile_mapping()
        
        # Map outputs to structures
        mapping_results = self.map_outputs_to_structure(outputs, repo_mapping, profile_mapping)
        
        # Generate comprehensive report
        report = {
            "timestamp": datetime.now().isoformat(),
            "rust_binary_status": rust_status,
            "discovered_outputs": outputs,
            "repository_structure": repo_mapping,
            "profile_structure": profile_mapping,
            "mapping_results": mapping_results,
            "execution_mode": "dry_run" if dry_run else "live",
            "recommendations": self.generate_recommendations(rust_status, outputs, mapping_results)
        }
        
        return report
    
    def generate_recommendations(self, rust_status: dict, outputs: dict, mapping_results: dict) -> list:
        """Generate recommendations for system optimization"""
        recommendations = []
        
        if not rust_status["binary_exists"]:
            recommendations.append("⚠️ Rust binary not found - compile intelligence-system-rust")
        
        if not rust_status["is_running"]:
            recommendations.append("🚀 Start Rust intelligence system for live data generation")
        
        if mapping_results["files_processed"] == 0:
            recommendations.append("📊 No intelligence output files found - check data generation")
        
        if len(outputs["advanced_system_outputs"]) > 0:
            recommendations.append("✅ Advanced intelligence system outputs detected - good integration")
        
        recommendations.append("🔄 Set up automated mapping via cronjob for continuous integration")
        recommendations.append("📱 Configure topic delivery automation for real-time reporting")
        
        return recommendations
    
    def save_mapping_configuration(self, report: dict) -> str:
        """Save mapping configuration for future reference"""
        timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
        config_file = f"rust_output_mapping_config_{timestamp}.json"
        
        with open(config_file, 'w') as f:
            json.dump(report, f, indent=2)
        
        return config_file

def main():
    """Main execution function"""
    print("🎯 Rust Binary Output Repository Mapping System")
    print("=" * 80)
    
    mapper = RustOutputMapper()
    
    # Execute mapping analysis
    report = mapper.execute_mapping(dry_run=True)
    
    # Save configuration
    config_file = mapper.save_mapping_configuration(report)
    
    # Display results
    print(f"📊 Mapping Analysis Complete!")
    print(f"📄 Configuration saved: {config_file}")
    print(f"🔍 Rust binary status: {'✅ Active' if report['rust_binary_status']['is_running'] else '⚠️ Not running'}")
    print(f"📁 Files discovered: {sum(len(v) for v in report['discovered_outputs'].values())}")
    print(f"🗂️ Repository structure: {len(report['repository_structure'])} categories")
    print(f"👤 Profile structure: {len(report['profile_structure'])} categories")
    print(f"🎯 Files processed: {report['mapping_results']['files_processed']}")
    
    print("\n💡 RECOMMENDATIONS:")
    for i, rec in enumerate(report['recommendations'], 1):
        print(f"  {i}. {rec}")
    
    print(f"\n✅ Rust Output Repository Mapping Framework Ready!")
    print(f"🔄 Ready for integration with cronjob system and topic delivery")

if __name__ == "__main__":
    main()