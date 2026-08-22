#!/usr/bin/env python3
"""
Social-Economic Correlation Analysis - Python Wrapper
Calls advanced Rust-based correlation engine for comprehensive analysis
"""

import subprocess
import sys
import os
import json
from datetime import datetime

def run_rust_correlation_analysis():
    """Execute Rust-based social-economic correlation analysis"""
    print("🚀 SOCIAL-ECONOMIC CORRELATION ANALYSIS")
    print("📊 Executing advanced Rust correlation engine...")
    
    try:
        # Path to Rust binary
        rust_binary = os.path.join(os.path.dirname(__file__), "commodity_collector_rust")
        
        if not os.path.exists(rust_binary):
            print(f"❌ Error: Rust binary not found at {rust_binary}")
            return False
            
        # Execute Rust correlation analysis
        print("⚡ Running high-performance correlation analysis...")
        result = subprocess.run([rust_binary], 
                              capture_output=True, 
                              text=True, 
                              timeout=30)
        
        if result.returncode == 0:
            print("✅ Rust correlation analysis completed successfully")
            print(f"📊 Output: {len(result.stdout)} characters")
            
            # Save analysis output
            timestamp = datetime.now().strftime("%Y%m%d_%H%M")
            output_file = f"social_economic_analysis_{timestamp}.txt"
            
            with open(output_file, 'w', encoding='utf-8') as f:
                f.write(f"Social-Economic Correlation Analysis\n")
                f.write(f"Generated: {datetime.now().isoformat()}\n")
                f.write(f"Engine: Rust-based correlation analysis\n\n")
                f.write(result.stdout)
            
            print(f"💾 Analysis saved to {output_file}")
            
            # Display key insights
            if result.stdout:
                print("\n" + "="*50)
                print("📈 CORRELATION ANALYSIS SUMMARY")
                print("="*50)
                print(result.stdout[:1000] + "..." if len(result.stdout) > 1000 else result.stdout)
            
            return True
            
        else:
            print(f"❌ Rust correlation analysis failed with exit code {result.returncode}")
            if result.stderr:
                print(f"Error: {result.stderr}")
            return False
            
    except subprocess.TimeoutExpired:
        print("❌ Rust correlation analysis timed out (30s limit)")
        return False
    except Exception as e:
        print(f"❌ Error running correlation analysis: {e}")
        return False

def main():
    """Main execution function"""
    print("🔥 Social-Economic Correlation Analysis System")
    print("🦀 Powered by Rust - High Performance Statistical Engine")
    print(f"📅 {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}")
    print("-" * 60)
    
    success = run_rust_correlation_analysis()
    
    if success:
        print("\n✅ Social-Economic Analysis completed successfully!")
        print("🚀 Rust-powered correlation matrices generated")
        print("📊 Portfolio analysis ready for decision-making")
    else:
        print("\n❌ Social-Economic Analysis failed")
        print("🔧 Check Rust binary availability and permissions")
        sys.exit(1)

if __name__ == "__main__":
    main()