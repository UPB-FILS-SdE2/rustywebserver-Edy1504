[![Review Assignment Due Date](https://classroom.github.com/assets/deadline-readme-button-24ddc0f5d75046c5622901739e7c5dd533143b0c8e959d652212380cedb1ea36.svg)](https://classroom.github.com/a/TXciPqtn)
# Rustwebserver

Detail the homework implementation.
runner modul: Acest modul conține probabil funcții (exec_run) și utilități (sigsegv_handler) necesare pentru executarea fișierului ELF. sigsegv_handler este un gestionar de semnale care se ocupă de defectele de segmentare (SIGSEGV), care sunt utilizate pentru a gestiona defectele de pagină în timpul accesărilor de memorie.

exec: Această funcție este responsabilă de execuția fișierului ELF. În mod normal, va efectua următoarele sarcini:

Parsarea ELF: Citește și analizează fișierul ELF pentru a extrage informații precum antetele programului (Elf32_Phdr).
Configurarea gestionarului de semnale: Înregistrați sigsegv_handler pentru a gestiona defectele de segmentare (SIGSEGV). Acest manipulator este esențial pentru punerea în aplicare a mecanismului de paginare la cerere descris.
Execuție: În cele din urmă, executați fișierul ELF utilizând funcții din modulul runner, cum ar fi exec_run.

main: Punctul principal de intrare al programului. Acesta inițializează structurile necesare, analizează argumentele din linia de comandă (omise în prezent în fragmentul de cod furnizat) și dă startul execuției fișierului ELF prin apelarea exec.

Parsing ELF: Funcția exec va utiliza un analizor (care nu este prezentat în întregime în fragmentul furnizat) pentru a citi antetele ELF și antetele de program (Elf32_Phdr). Aceste antete descriu segmente ale fișierului ELF, inclusiv secțiunile de cod, date și stivă.

Gestionarea semnalelor: sigsegv_handler interceptează defectele de segmentare (SIGSEGV) declanșate de accesările de memorie. Gestionează maparea și protecția memoriei utilizând funcții precum mmap și mprotect pentru a se asigura că accesările memoriei sunt gestionate în conformitate cu permisiunile specificate în anteturile fișierelor ELF (r-x, rw- etc.).

Execuție (exec_run): După configurarea fișierului ELF (încărcarea segmentelor în memorie), funcția principală va apela eventual exec_run din modulul runner. Această funcție stabilește mediul inițial de execuție (cum ar fi configurarea stivei și variabilele de mediu), apoi sare la punctul de intrare (ehdr.e_entry) specificat în antetul fișierului ELF.