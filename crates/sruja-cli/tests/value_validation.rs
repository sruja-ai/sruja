mod common;
use common::*;

#[test]
fn test_clean_express_saas_score_and_map() {
    let repo = create_test_repo();
    let root = repo.path();

    // 1. Setup clean repo
    write_file(
        root,
        "src/users/controller.ts",
        r#"
import { UserService } from './service';
export class UserController {
  constructor(private userService: UserService) {}
  async getUser(id: string) { return this.userService.findById(id); }
}
"#,
    );
    write_file(
        root,
        "src/users/service.ts",
        r#"
import { PrismaClient } from '../database';
export class UserService {
  constructor(private prisma: PrismaClient) {}
  async findById(id: string) { return this.prisma.user.findUnique({ where: { id } }); }
}
"#,
    );
    write_file(
        root,
        "src/billing/service.ts",
        r#"
import { UserService } from '../users/service';
export class BillingService {
  constructor(private userService: UserService) {}
  async bill(userId: string) { return this.userService.findById(userId); }
}
"#,
    );
    write_file(
        root,
        "src/database.ts",
        r#"
export class PrismaClient { user = { findUnique: async (args: any) => ({ id: '1', name: 'Test' }) } }
"#,
    );

    // 2. Run sruja
    let (success, stdout, _stderr) =
        run_sruja(&["quickstart", "-r", root.to_str().unwrap(), "-f", "text"]);
    assert!(success, "Sruja should succeed on clean repo");

    // 3. Health Score should be 100
    assert!(
        stdout.contains("Health Score (structural only): 100/100"),
        "Clean project should have 100/100 score. Output: {}",
        stdout
    );

    // 4. Domain Map should show src/users, src/billing
    assert!(
        stdout.contains("src/users") || stdout.contains("users"),
        "Domain map should show users domain. Output: {}",
        stdout
    );
    assert!(
        stdout.contains("src/billing") || stdout.contains("billing"),
        "Domain map should show billing domain. Output: {}",
        stdout
    );

    // 5. No findings
    assert!(
        stdout.contains("No critical issues found"),
        "Clean project should have no findings. Output: {}",
        stdout
    );
}

#[test]
fn test_messy_express_saas_score_and_findings() {
    let repo = create_test_repo();
    let root = repo.path();

    // 1. Setup messy repo (Circular Dep: Users <-> Billing)
    write_file(
        root,
        "src/users/service.ts",
        r#"
import { BillingService } from '../billing/service';
export class UserService { constructor(private billing: BillingService) {} }
"#,
    );
    write_file(
        root,
        "src/billing/service.ts",
        r#"
import { UserService } from '../users/service';
export class BillingService { constructor(private user: UserService) {} }
"#,
    );

    // 2. God Module (15 distinct imports)
    for i in 1..=12 {
        write_file(
            root,
            &format!("src/users/dep{}.ts", i),
            &format!("export const d{} = () => {{}};", i),
        );
    }
    write_file(
        root,
        "src/users/god_service.ts",
        r#"
import { UserService } from './service';
import { BillingService } from '../billing/service';
import { PrismaClient } from '../database/index';
import { d1 } from './dep1'; import { d2 } from './dep2'; import { d3 } from './dep3';
import { d4 } from './dep4'; import { d5 } from './dep5'; import { d6 } from './dep6';
import { d7 } from './dep7'; import { d8 } from './dep8'; import { d9 } from './dep9';
import { d10 } from './dep10'; import { d11 } from './dep11'; import { d12 } from './dep12';
export class GodService { constructor(private u: UserService, private b: BillingService, private d: PrismaClient) {
  console.log(d1, d2, d3, d4, d5, d6, d7, d8, d9, d10, d11, d12);
} }
"#,
    );
    write_file(
        root,
        "src/database/index.ts",
        "export class PrismaClient {}",
    );

    // 3. Layer Violation (UI -> DB)
    write_file(
        root,
        "src/ui/UserUI.tsx",
        r#"
import { PrismaClient } from '../database/index';
export const UserUI = () => { const p = new PrismaClient(); return null; }
"#,
    );

    // 4. Run sruja
    let (success, stdout, _stderr) =
        run_sruja(&["quickstart", "-r", root.to_str().unwrap(), "-f", "text"]);
    assert!(success, "Sruja should succeed on messy repo");

    // 5. Assertions
    assert!(
        !stdout.contains("100/100"),
        "Messy project should NOT have 100/100 score. Output: {}",
        stdout
    );
    assert!(
        stdout.contains("Circular dependency detected"),
        "Should detect circular dependency. Output: {}",
        stdout
    );
    assert!(
        stdout.contains("Bottleneck Detected") && stdout.contains("God Module"),
        "Should detect God Module. Output: {}",
        stdout
    );
    assert!(
        stdout.contains("Top targets:"),
        "Suggestion should include top targets. Output: {}",
        stdout
    );
    assert!(
        stdout.contains("Layer violation"),
        "Should detect layer violation (UI -> DB). Output: {}",
        stdout
    );
}
