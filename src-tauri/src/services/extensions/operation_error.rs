use super::error_codes;

#[derive(Clone, Copy)]
pub enum Operation {
    InstallGit,
    InstallNpm,
    Update,
    Uninstall,
    Cleanup,
}

impl Operation {
    fn label(self) -> &'static str {
        match self {
            Self::InstallGit => "install_git",
            Self::InstallNpm => "install_npm",
            Self::Update => "update",
            Self::Uninstall => "uninstall",
            Self::Cleanup => "cleanup",
        }
    }

    fn fallback(self) -> &'static str {
        match self {
            Self::InstallGit | Self::InstallNpm => error_codes::INSTALL_FAILED,
            Self::Update => error_codes::UPDATE_FAILED,
            Self::Uninstall => error_codes::UNINSTALL_FAILED,
            Self::Cleanup => error_codes::CLEANUP_FAILED,
        }
    }
}

pub fn report(operation: Operation, internal: &str) -> String {
    let code = classify(operation, internal);
    super::operation_log::write(operation.label(), code, internal_reason(internal));
    code.to_string()
}

fn classify(operation: Operation, error: &str) -> &'static str {
    if error.contains("Nombre maximal") {
        return error_codes::LIMIT_REACHED;
    }
    if error.contains("déjà enregistrée") {
        return error_codes::ALREADY_INSTALLED;
    }
    if error.contains("identité") {
        return error_codes::UPDATE_IDENTITY_CHANGED;
    }
    if error.contains("n'est pas gérée") || error.contains("non gérée") {
        return error_codes::UPDATE_UNAVAILABLE;
    }
    if error.contains("URL Git")
        || error.contains("Référence Git")
        || error.contains("Source d'extension invalide")
    {
        return error_codes::SOURCE_INVALID;
    }
    if error.contains("Package npm")
        || error.contains("package npm")
        || error.contains("Version ou tag npm")
        || error.contains("Nom de package npm")
    {
        return error_codes::PACKAGE_INVALID;
    }
    if error.contains("Téléchargement Git") || error.contains("Révision Git") {
        if error.contains("expiré") {
            return error_codes::GIT_TIMEOUT;
        }
        return error_codes::GIT_DOWNLOAD_FAILED;
    }
    if error.contains("Runtime Node.js") || error.contains("Gestionnaire npm") {
        return error_codes::RUNTIME_UNAVAILABLE;
    }
    if error.contains("Commande d'installation") {
        return error_codes::DEPENDENCY_INSTALL_FAILED;
    }
    if error.contains("Manifeste")
        || error.contains("Point d'entrée")
        || error.contains("Installation d'extension")
        || error.contains("Package d'extension")
        || error.contains("Source d'extension introuvable")
    {
        return error_codes::MANIFEST_INVALID;
    }
    if error.contains("Stockage")
        || error.contains("Cache npm")
        || error.contains("Configuration npm impossible")
        || error.contains("nettoyer")
        || error.contains("Nettoyage")
        || error.contains("Suppression des fichiers")
    {
        return error_codes::STORAGE_FAILED;
    }
    operation.fallback()
}

fn internal_reason(error: &str) -> &'static str {
    if error.contains("expiré") {
        "timeout"
    } else if error.contains("indisponible") {
        "unavailable"
    } else if error.contains("invalide") || error.contains("invalid") {
        "invalid_input_or_content"
    } else if error.contains("introuvable") {
        "missing"
    } else if error.contains("déjà enregistrée") {
        "duplicate"
    } else if error.contains("Nombre maximal") || error.contains("Trop de") {
        "limit"
    } else if error.contains("identité") {
        "identity_changed"
    } else if error.contains("nettoyer") || error.contains("Nettoyage") {
        "cleanup"
    } else if error.contains("interrompue") {
        "interrupted"
    } else {
        "operation_failed"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exposes_specific_safe_codes_without_forwarding_internal_details() {
        assert_eq!(
            classify(Operation::InstallGit, "Téléchargement Git impossible."),
            error_codes::GIT_DOWNLOAD_FAILED
        );
        assert_eq!(
            classify(Operation::InstallNpm, "Commande d'installation expirée."),
            error_codes::DEPENDENCY_INSTALL_FAILED
        );
        assert_eq!(
            classify(
                Operation::Update,
                "L'identité de l'extension mise à jour a changé."
            ),
            error_codes::UPDATE_IDENTITY_CHANGED
        );
        assert_eq!(
            internal_reason("Commande d'installation expirée."),
            "timeout"
        );
    }
}
