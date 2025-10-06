from setuptools import setup, find_packages

setup(
    name="cocoatly",
    version="0.1.0",
    packages=find_packages(),
    install_requires=[
        "click>=8.1.7",
        "requests>=2.31.0",
        "pyyaml>=6.0.1",
        "rich>=13.7.0",
        "jsonschema>=4.20.0",
        "packaging>=23.2",
        "python-dateutil>=2.8.2",
        "tabulate>=0.9.0",
    ],
    entry_points={
        "console_scripts": [
            "cocoatly=cocoatly.cli.main:cli",
        ],
    },
    python_requires=">=3.9",
    author="Cocoatly Team",
    description="A high-performance package manager built with Rust and Python",
    long_description=open("README.md").read() if __file__ else "",
    long_description_content_type="text/markdown",
    license="MIT",
    classifiers=[
        "Development Status :: 4 - Beta",
        "Intended Audience :: Developers",
        "License :: OSI Approved :: MIT License",
        "Programming Language :: Python :: 3.9",
        "Programming Language :: Python :: 3.10",
        "Programming Language :: Python :: 3.11",
        "Programming Language :: Python :: 3.12",
    ],
)
