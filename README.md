# FPGA Agilex 5 - Secure Boot Demo

## Project Description

This project demonstrates the process of loading bitstream images (SOF files) onto an **Intel Agilex 5 FPGA** and verifying their integrity using **Secure Boot** functionality. The demo application allows the user to load two different SOF images via a simple Windows GUI, and after loading the image, the FPGA verifies its validity by checking its signature.

### Key Features

- **Bitstream Upload**: The demo application provides an easy-to-use Windows interface for uploading SOF files to the FPGA. 
- **Secure Boot Verification**: After the FPGA loads the bitstream, it validates the integrity of the image by analyzing its digital signature, ensuring the authenticity of the file.
- **Demonstration of Secure Boot**: This showcases the Secure Boot process in Intel Agilex 5 FPGAs, providing enhanced security by verifying the integrity of the bitstream before execution.

## Setup Instructions

### Requirements

- **Intel Agilex 5 FPGA**
- **Windows Operating System** with the ability to communicate with the FPGA hardware
- **Intel Quartus Prime Software** for working with the FPGA (if needed for programming)
- **USB-Blaster or similar programming hardware** (if needed to interface with the FPGA)

### Steps to Run the Demo

1. **Install Dependencies**: Make sure you have the necessary drivers and programming software (Intel Quartus Prime, USB-Blaster, etc.) installed on your Windows system.
2. **Download the Demo Application**: Clone this repository to your local machine.
3. **Load SOF Files**: Use the demo application to load the appropriate SOF files into the FPGA. 
4. **Start the Demo**: Once the image is loaded, the FPGA will automatically verify the signature of the loaded image to ensure its authenticity.
5. **Observe Output**: After successful loading, the FPGA will either proceed with the execution of the image if the signature is correct, or it will notify an error if the signature is invalid.

## Visual Demonstration

The demo includes two images (SOF files) that can be loaded into the FPGA through the Windows interface. These images are verified by the FPGA, which checks their digital signatures before executing the bitstream.
