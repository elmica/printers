#!/usr/bin/env python3
"""
Create a valid Windows ICO file from PNG files
This script creates a proper ICO file that Windows Resource Compiler can use
"""
import sys
import struct

def create_ico_from_png(png_path, ico_path):
    """Create a basic ICO file from PNG data"""
    try:
        with open(png_path, 'rb') as f:
            png_data = f.read()
        
        # ICO file structure:
        # ICO Header: Reserved (2) + Type (2) + Count (2) = 6 bytes
        # Image Entry: Width (1) + Height (1) + Colors (1) + Reserved (1) + 
        #              Planes (2) + Bits per pixel (2) + Size (4) + Offset (4) = 16 bytes
        # PNG Data follows
        
        # Check if it's a valid PNG
        if png_data[:8] != b'\x89PNG\r\n\x1a\n':
            print(f"Error: {png_path} is not a valid PNG file", file=sys.stderr)
            return False
        
        # Create ICO file
        ico_data = bytearray()
        
        # ICO Header
        ico_data.extend(struct.pack('<HHH', 0, 1, 1))  # Reserved, Type (1=ICO), Count (1 image)
        
        # Determine dimensions from PNG IHDR
        width = png_data[16]
        height = png_data[20]
        
        # Image Entry
        ico_data.append(width if width < 256 else 0)  # Width (0 = 256)
        ico_data.append(height if height < 256 else 0)  # Height (0 = 256)
        ico_data.append(0)  # Color palette (0 = no palette)
        ico_data.append(0)  # Reserved
        ico_data.extend(struct.pack('<HH', 1, 32))  # Planes (1) + Bits per pixel (32)
        ico_data.extend(struct.pack('<I', len(png_data)))  # Size of PNG data
        ico_data.extend(struct.pack('<I', 22))  # Offset to image data (6 + 16 = 22)
        
        # PNG data
        ico_data.extend(png_data)
        
        with open(ico_path, 'wb') as f:
            f.write(ico_data)
        
        print(f"Created {ico_path} from {png_path} ({len(ico_data)} bytes)")
        return True
    except Exception as e:
        print(f"Error creating ICO: {e}", file=sys.stderr)
        return False

if __name__ == '__main__':
    import os
    
    # Get script directory
    script_dir = os.path.dirname(os.path.abspath(__file__))
    
    # Use 32x32.png as source
    png_source = os.path.join(script_dir, '32x32.png')
    ico_output = os.path.join(script_dir, 'icon.ico')
    
    if not os.path.exists(png_source):
        print(f"Error: {png_source} not found", file=sys.stderr)
        sys.exit(1)
    
    if create_ico_from_png(png_source, ico_output):
        sys.exit(0)
    else:
        sys.exit(1)

