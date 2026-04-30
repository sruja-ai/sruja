# Support and Community

Sruja is an **open source project** maintained by the community. This document outlines the support options available for organizations and individuals using Sruja.

---

## Community Support (Free)

Sruja is primarily supported through community channels. All users have access to:

### Documentation
- **README.md** – Quick start guide and CLI reference
- **docs/** – Detailed documentation including installation, getting started guides, and language specifications
- **docs/KNOWN_LIMITATIONS.md** – Known limitations and scope constraints
- **book/** – Comprehensive documentation built with mdBook

### Community Channels
- **GitHub Discussions** – For questions, feature requests, and discussions
  - [github.com/sruja-ai/sruja/discussions](https://github.com/sruja-ai/sruja/discussions)
- **GitHub Issues** – For bug reports and technical issues
  - [github.com/sruja-ai/sruja/issues](https://github.com/sruja-ai/sruja/issues)

### Contributing
- Contributions of all sizes are welcome!
- See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines
- Report bugs, fix typos, add examples, or submit PRs for new features

### Self-Service Resources
- **Open Source Code** – Full source code available under Apache 2.0 license
- **Build from Source** – Build and modify the CLI and tools yourself
- **Issue Tracking** – View and track existing issues and feature requests

---

## No Commercial Support or SLA

**Important:** Sruja does not currently offer:

- ❌ Commercial support contracts
- ❌ Service Level Agreements (SLAs)
- ❌ Priority support for paying customers
- ❌ Dedicated support team or phone support
- ❌ Guaranteed response times
- ❌ Custom development services

All support is provided on a **best-effort community basis**. Response times depend on community availability.

---

## For Enterprise Organizations

If you're evaluating Sruja for enterprise use, consider the following:

### What We Offer
- ✅ Open source code you can audit and modify
- ✅ Apache 2.0 license (permissive, commercial-friendly)
- ✅ Active community and regular releases
- ✅ Comprehensive documentation
- ✅ GitHub Discussions for Q&A
- ✅ Transparent development process

### What We Don't Offer
- ❌ Guaranteed support response times
- ❌ SLA-backed uptime
- ❌ Paid support packages
- ❌ Enterprise security certifications
- ❌ Dedicated account management

### Recommendations for Enterprise Adoption

If your organization requires:

1. **Guaranteed Support Response Times**
   - Consider hiring an internal team member to become a Sruja expert
   - Build internal expertise through self-service documentation
   - Budget for internal maintenance and support

2. **SLA or Uptime Guarantees**
   - Sruja is a CLI tool run locally or in CI, not a hosted service
   - You control the environment and deployment
   - Self-hosting eliminates external uptime dependencies

3. **Security Certifications**
   - Review the open source code for security assessment
   - Conduct your own security audits
   - Contribute security fixes back to the project

4. **Custom Development**
   - Fork the repository for custom modifications
   - Build custom tooling on top of Sruja's CLI primitives
   - Contribute useful features upstream for community benefit

---

## Support Best Practices

### Getting Help Quickly

1. **Search first** – Check GitHub Issues and Discussions for similar questions
2. **Be specific** – Include error messages, CLI commands, and context
3. **Use documentation** – Review docs/ and book/ for answers to common questions
4. **Report bugs properly** – Use the issue template and provide reproduction steps

### Production Use

When using Sruja in production:

- **Pin versions** – Use specific versions in CI/CD, not `latest`
- **Test before deploying** – Run `just test-cli-smoke` after updates
- **Monitor issues** – Watch GitHub Issues for known problems in your version
- **Have a rollback plan** – Know how to revert if a version introduces issues

### Contributing Back

If your organization uses Sruja successfully:

- Share success stories in GitHub Discussions
- Contribute bug fixes for issues you encounter
- Add documentation for your use cases
- Sponsor the project or contributors if you find value

---

## Future Support Options

The Sruja team may explore commercial support options in the future based on community demand and organizational needs. If your organization would be interested in:

- Paid support packages
- SLA-backed support
- Priority issue handling
- Custom development contracts

Please start a **Discussion** on GitHub to express interest. This helps us understand demand and plan accordingly.

---

## Summary

| Support Type | Availability | Response Time | Cost |
|-------------|---------------|----------------|-------|
| **GitHub Discussions** | ✅ Available | Community-dependent (hours to days) | Free |
| **GitHub Issues** | ✅ Available | Community-dependent | Free |
| **Documentation** | ✅ Available | Immediate | Free |
| **Source Code** | ✅ Available | Immediate | Free |
| **Commercial Support** | ❌ Not available | N/A | N/A |
| **SLA** | ❌ Not available | N/A | N/A |

**Bottom line:** Sruja is community-supported open source software. Use it if you're comfortable with self-service support, or budget for internal expertise if you need guaranteed support.

---

## Questions?

- **For questions:** Use [GitHub Discussions](https://github.com/sruja-ai/sruja/discussions)
- **For bugs:** Use [GitHub Issues](https://github.com/sruja-ai/sruja/issues)
- **For feature requests:** Use [GitHub Discussions](https://github.com/sruja-ai/sruja/discussions) or open an issue

We appreciate your interest in Sruja and welcome contributions from the community!
