#!/usr/bin/env python3
"""
DevRC DSL Interpreter
Parses and executes .devrc configuration files
"""

import re
import os
import json
import subprocess
from pathlib import Path
from typing import Dict, List, Any, Optional


class DevRCInterpreter:
    def __init__(self):
        self.variables = {}
        self.sections = {}
        self.section_types = {}
        self.current_section = None
        self.imported_files = set()
        self.import_stack = []
        
    def parse_file(self, filepath: str) -> Dict[str, List[str]]:
        """Parse a .devrc file into sections"""
        # Prevent circular imports
        abs_path = os.path.abspath(filepath)
        if abs_path in self.import_stack:
            print(f"✗ Circular import detected: {filepath}")
            return {}
        
        self.import_stack.append(abs_path)
        
        with open(filepath, 'r') as f:
            content = f.read()
        
        sections = {}
        current_section = None
        current_type = None
        
        for line in content.split('\n'):
            # Remove comments
            if '#' in line:
                line = line.split('#')[0]
            
            line = line.strip()
            if not line:
                continue
            
            # Check for imports
            if line.startswith('@DEVRC.IMPORT.'):
                self.handle_import(line, os.path.dirname(filepath))
                continue
            
            # Check for type annotations
            if line.startswith('@[') and line.endswith(']'):
                current_type = line[2:-1]
                print(f"✓ Type annotation found: {current_type}")
                continue
                
            # Check for section headers
            if line.startswith('[') and line.endswith(']'):
                current_section = line[1:-1]
                sections[current_section] = []
                if current_type:
                    self.section_types[current_section] = current_type
                    current_type = None
            elif current_section:
                sections[current_section].append(line)
        
        self.import_stack.pop()
        return sections
    
    def tokenize(self, line: str) -> List[str]:
        """Tokenize a line into components"""
        # Handle quoted strings
        tokens = []
        current = []
        in_quotes = False
        
        for char in line:
            if char == '"':
                in_quotes = not in_quotes
                current.append(char)
            elif char in [' ', '\t'] and not in_quotes:
                if current:
                    tokens.append(''.join(current))
                    current = []
            else:
                current.append(char)
        
        if current:
            tokens.append(''.join(current))
        
        return tokens
    
    def parse_assignment(self, line: str) -> Optional[tuple]:
        """Parse variable assignment"""
        if '=' in line:
            parts = line.split('=', 1)
            var_name = parts[0].strip()
            var_value = parts[1].strip()
            return (var_name, var_value)
        return None
    
    def evaluate_expression(self, expr: str) -> Any:
        """Evaluate an expression"""
        expr = expr.strip()
        
        # Remove quotes
        if expr.startswith('"') and expr.endswith('"'):
            return expr[1:-1]
        
        # Check if it's a variable reference
        if expr in self.variables:
            return self.variables[expr]
        
        # Check for boolean literals
        if expr.lower() in ['true', '-true']:
            return True
        if expr.lower() in ['false', '-false']:
            return False
        
        # Check for null
        if expr.lower() == 'null':
            return None
        
        return expr
    
    def handle_import(self, line: str, base_path: str):
        """Handle @DEVRC.IMPORT.[variablename] statements"""
        # Parse import statement: @DEVRC.IMPORT.[variablename] or @DEVRC.IMPORT.[variablename]="path"
        match = re.match(r'@DEVRC\.IMPORT\.(\w+)(?:="?([^"]+)"?)?', line)
        if not match:
            print(f"✗ Invalid import syntax: {line}")
            return
        
        var_name = match.group(1)
        import_path = match.group(2)
        
        # If no path specified, check if variable exists
        if not import_path:
            if var_name not in self.variables:
                print(f"✗ Import failed: variable '{var_name}' not defined")
                return
            import_path = self.variables[var_name]
        
        # Resolve relative paths
        if not os.path.isabs(import_path):
            import_path = os.path.join(base_path, import_path)
        
        # Check if already imported
        abs_import_path = os.path.abspath(import_path)
        if abs_import_path in self.imported_files:
            print(f"✓ Already imported: {import_path}")
            return
        
        # Check if file exists
        if not os.path.exists(import_path):
            print(f"✗ Import file not found: {import_path}")
            return
        
        print(f"✓ Importing from: {import_path}")
        self.imported_files.add(abs_import_path)
        
        # Parse and merge the imported file
        imported_sections = self.parse_file(import_path)
        for section_name, lines in imported_sections.items():
            if section_name in self.sections:
                # Merge with existing section
                print(f"  ↳ Merging section: [{section_name}]")
                self.sections[section_name].extend(lines)
            else:
                # Add new section
                print(f"  ↳ Adding section: [{section_name}]")
                self.sections[section_name] = lines
                # Copy type if exists
                if section_name in self.section_types:
                    self.section_types[section_name] = self.section_types[section_name]
    
    def create_folder(self, path: str):
        """Create a folder if it doesn't exist"""
        path = path.strip('"').replace('*', '')
        try:
            Path(path).mkdir(parents=True, exist_ok=True)
            print(f"✓ Created folder: {path}")
        except Exception as e:
            print(f"✗ Error creating folder {path}: {e}")
    
    def output_to_file(self, path: str, content: Any = None):
        """Handle output to file"""
        path = path.strip('"').replace('*', '')
        try:
            if '*' in path or path.endswith('/'):
                # Directory output
                Path(path).mkdir(parents=True, exist_ok=True)
                print(f"✓ Prepared output directory: {path}")
            else:
                # File output
                Path(path).parent.mkdir(parents=True, exist_ok=True)
                if content:
                    with open(path, 'w') as f:
                        if isinstance(content, dict):
                            json.dump(content, f, indent=2)
                        else:
                            f.write(str(content))
                print(f"✓ Output to: {path}")
        except Exception as e:
            print(f"✗ Error outputting to {path}: {e}")
    
    def execute_command(self, command: List[str]):
        """Execute a system command"""
        try:
            result = subprocess.run(command, capture_output=True, text=True)
            print(f"✓ Executed: {' '.join(command)}")
            return result.stdout
        except Exception as e:
            print(f"✗ Error executing command: {e}")
            return None
    
    def process_line(self, line: str):
        """Process a single line of DevRC code"""
        tokens = self.tokenize(line)
        if not tokens:
            return
        
        # Handle assignments
        assignment = self.parse_assignment(line)
        if assignment:
            var_name, var_value = assignment
            self.variables[var_name] = self.evaluate_expression(var_value)
            print(f"✓ Set {var_name} = {self.variables[var_name]}")
            return
        
        # Handle .devrc commands
        if tokens[0] == '.devrc':
            self.handle_devrc_command(tokens[1:])
        
        # Handle if statements
        elif tokens[0] == 'if':
            self.handle_if_statement(line)
        
        # Handle for loops
        elif tokens[0] == 'for':
            self.handle_for_loop(line)
        
        # Handle do statements
        elif tokens[0] == 'do':
            self.handle_do_statement(tokens[1:])
        
        # Handle out command
        elif tokens[0] == 'out':
            if len(tokens) > 1:
                self.output_to_file(tokens[1])
    
    def handle_devrc_command(self, tokens: List[str]):
        """Handle .devrc specific commands"""
        i = 0
        while i < len(tokens):
            token = tokens[i]
            
            if token == '-out' and i + 1 < len(tokens):
                self.output_to_file(tokens[i + 1])
                i += 2
            
            elif token == '-crfolder' and i + 1 < len(tokens):
                self.create_folder(tokens[i + 1])
                i += 2
            
            elif token == '-pop':
                if i + 1 < len(tokens):
                    print(f"✓ Pop operation: {tokens[i + 1]}")
                i += 2
            
            elif token == '-plugin':
                print("✓ Plugin mode enabled")
                i += 1
            
            elif token == '-config':
                print("✓ Config mode enabled")
                i += 1
            
            elif token == '-c':
                print("✓ Compile mode enabled")
                i += 1
            
            else:
                i += 1
    
    def handle_if_statement(self, line: str):
        """Handle if statements"""
        # Extract condition
        match = re.search(r'if \((.*?)\) is (.*?)(?:\s+do\s+|\s+|$)', line)
        if match:
            var_name = match.group(1).strip()
            expected = match.group(2).strip()
            
            var_value = self.variables.get(var_name, False)
            expected_value = self.evaluate_expression(expected)
            
            if var_value == expected_value:
                # Execute the rest of the line
                rest = line[match.end():].strip()
                if rest:
                    print(f"✓ Condition met: {var_name} is {expected_value}")
                    self.process_line(rest)
            else:
                print(f"✗ Condition not met: {var_name} is not {expected_value}")
    
    def handle_for_loop(self, line: str):
        """Handle for loops"""
        match = re.search(r'for \((.*?)\)', line)
        if match:
            var_name = match.group(1).strip()
            print(f"✓ For loop over: {var_name}")
            # Execute the rest of the line
            rest = line[match.end():].strip()
            if rest:
                self.process_line(rest)
    
    def handle_do_statement(self, tokens: List[str]):
        """Handle do statements"""
        print(f"✓ Do statement: {' '.join(tokens)}")
        self.handle_devrc_command(tokens)
    
    def execute_section(self, section_name: str):
        """Execute a specific section"""
        if section_name not in self.sections:
            print(f"✗ Section not found: {section_name}")
            return
        
        section_type = self.section_types.get(section_name, "untyped")
        print(f"\n=== Executing section: {section_name} @[{section_type}] ===")
        for line in self.sections[section_name]:
            self.process_line(line)
    
    def execute_all(self):
        """Execute all sections in order"""
        for section_name, lines in self.sections.items():
            self.execute_section(section_name)
    
    def run(self, filepath: str, sections: Optional[List[str]] = None):
        """Run the DevRC interpreter"""
        print(f"DevRC Interpreter - Loading {filepath}")
        self.sections = self.parse_file(filepath)
        
        print(f"\n✓ Total sections loaded: {len(self.sections)}")
        print(f"✓ Total imports processed: {len(self.imported_files)}")
        
        if sections:
            for section in sections:
                self.execute_section(section)
        else:
            self.execute_all()
        
        print("\n=== Execution complete ===")
        if self.imported_files:
            print(f"Imported files:")
            for imp in self.imported_files:
                print(f"  - {imp}")


def main():
    import argparse
    
    parser = argparse.ArgumentParser(description='DevRC DSL Interpreter')
    parser.add_argument('file', help='.devrc file to execute')
    parser.add_argument('--section', '-s', action='append', 
                       help='Specific section(s) to execute')
    parser.add_argument('--dry-run', '-d', action='store_true',
                       help='Parse without executing')
    
    args = parser.parse_args()
    
    interpreter = DevRCInterpreter()
    
    if args.dry_run:
        sections = interpreter.parse_file(args.file)
        print("Parsed sections:")
        for name, lines in sections.items():
            section_type = interpreter.section_types.get(name, "untyped")
            print(f"\n@[{section_type}]")
            print(f"[{name}]")
            for line in lines:
                print(f"  {line}")
    else:
        interpreter.run(args.file, args.section)


if __name__ == '__main__':
    main()