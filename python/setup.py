from setuptools import setup, find_packages

setup(
    name="streamregex",
    version="0.1.0",
    packages=find_packages(),
    install_requires=[
        "numpy>=1.20.0",
    ],
    author="Mark Wernsdorfer",
    author_email="wernsdorfer@gmail.com",
    description="High-performance pattern matching library for streaming data",
    long_description=open("README.md").read(),
    long_description_content_type="text/markdown",
    url="https://github.com/wehnsdaefflae/streamregex",
    classifiers=[
        "Development Status :: 4 - Beta",
        "Intended Audience :: Developers",
        "License :: OSI Approved :: MIT License",
        "Programming Language :: Python :: 3.13",
        "Programming Language :: Rust",
        "Topic :: Security",
        "Topic :: Software Development :: Libraries :: Python Modules",
    ],
    python_requires=">=3.13",
)